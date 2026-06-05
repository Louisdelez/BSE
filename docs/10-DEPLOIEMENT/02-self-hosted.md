# 10.02 — Déploiement self-hosted

> Le path *gold* de BSE : tu installes en une commande.

## Docker Compose (recommandé)

### Fichier `docker-compose.yml` minimal

```yaml
version: "3.9"
services:
  bse-server:
    image: ghcr.io/bse-app/bse-server:latest
    environment:
      BSE_DATABASE_URL: postgres://bse:secret@postgres:5432/bse
      BSE_S3_ENDPOINT: http://minio:9000
      BSE_S3_BUCKET: bse
      BSE_S3_ACCESS_KEY: bse
      BSE_S3_SECRET_KEY: secret
      BSE_JWT_SECRET: change-me-in-production-${RANDOM}
      BSE_PUBLIC_URL: https://bse.example.com
    depends_on:
      - postgres
      - minio
    restart: unless-stopped

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: bse
      POSTGRES_PASSWORD: secret
      POSTGRES_DB: bse
    volumes:
      - postgres-data:/var/lib/postgresql/data
    restart: unless-stopped

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: bse
      MINIO_ROOT_PASSWORD: secret
    volumes:
      - minio-data:/data
    restart: unless-stopped

  caddy:
    image: caddy:2-alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy-data:/data
      - caddy-config:/config
    restart: unless-stopped

volumes:
  postgres-data:
  minio-data:
  caddy-data:
  caddy-config:
```

### `Caddyfile`

```caddyfile
bse.example.com {
    reverse_proxy bse-server:8080
    encode gzip
    header {
        Strict-Transport-Security "max-age=31536000"
        X-Content-Type-Options "nosniff"
        Referrer-Policy "no-referrer"
    }
}
```

### Lancement

```bash
git clone https://github.com/bse-app/bse-deploy
cd bse-deploy
cp .env.example .env
# Éditer .env (domain, secrets)
docker compose up -d
```

Et c'est tout. Caddy gère le cert Let's Encrypt automatiquement.

## Kubernetes

Pour des déploiements plus structurés, un **Helm chart** sera fourni.

```bash
helm repo add bse https://charts.bse-app.io
helm install bse bse/bse-server \
  --set domain=bse.example.com \
  --set postgres.password=secret \
  --set s3.endpoint=https://minio.local
```

Le chart inclut :
- Deployment BSE server (replica 2+)
- Service ClusterIP
- Ingress avec cert-manager
- ConfigMap pour la config
- Secrets pour creds
- PVC pour MinIO si embedded
- ServiceMonitor pour Prometheus

## Bare metal (sans Docker)

Pour les puristes :

```bash
# Build
cargo build --release -p bse-server

# Run
./target/release/bse-server --config /etc/bse/config.toml
```

### systemd unit
```ini
[Unit]
Description=BSE Server
After=network.target postgresql.service

[Service]
Type=simple
User=bse
WorkingDirectory=/opt/bse
ExecStart=/opt/bse/bse-server --config /etc/bse/config.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## Configuration

### Fichier `config.toml`

```toml
[server]
bind = "0.0.0.0:8080"
public_url = "https://bse.example.com"

[database]
url = "postgres://bse:password@localhost:5432/bse"
pool_size = 20

[storage]
backend = "s3"  # ou "filesystem"
s3_endpoint = "https://minio.local"
s3_bucket = "bse"
s3_access_key = "..."
s3_secret_key = "..."
s3_region = "auto"

[auth]
mode = "local"  # ou "oidc"
jwt_secret = "..."  # ou jwt_private_key_path pour RS256

[auth.oidc]
issuer = "https://accounts.google.com"
client_id = "..."
client_secret = "..."

[limits]
max_rooms = 1000
max_peers_per_room = 50
max_ops_per_sec_per_peer = 100
max_asset_size_mb = 20
max_project_size_gb = 1

[logging]
level = "info"  # debug, info, warn, error
format = "json"  # ou "text"

[metrics]
enable_prometheus = true
prometheus_path = "/metrics"
```

### Override via env vars

Toute config peut être surchargée par env var :
- `BSE_DATABASE_URL` → `[database] url`
- `BSE_AUTH_OIDC_CLIENT_ID` → `[auth.oidc] client_id`
- etc.

## Backups

### Postgres
- Daily `pg_dump`
- WAL archiving pour PITR (v1.x)

### MinIO / S3
- Versioning activé sur le bucket
- Lifecycle policy : ancien snapshots → glacier

### Outil `bse-backup` (CLI)
```bash
bse-backup create --dest s3://backups/2026-06-05.tar
bse-backup list
bse-backup restore --src s3://backups/2026-06-05.tar --confirm
```

## Upgrades

### Pattern recommandé
1. `docker compose pull`
2. `docker compose up -d`
3. Migrations DB appliquées automatiquement au démarrage
4. Zero downtime si plusieurs instances

### Breaking changes
Les release notes spécifient les actions manuelles éventuelles (rare).

## Monitoring

### Endpoints
- `/health` : healthcheck (200 si OK)
- `/ready` : readiness (200 si DB + S3 OK)
- `/metrics` : Prometheus

### Métriques clés à surveiller
- `bse_active_rooms`
- `bse_active_peers`
- `bse_http_requests_total{status}` 
- `bse_ws_messages_total{direction}`
- `bse_db_connections_active`
- `bse_db_query_duration_seconds`

### Logs
- Format JSON (parsable par Loki, Splunk, etc.)
- Niveau configurable
- Pas de PII inutile

## Sécurité de l'installation

### Checklist
- [ ] HTTPS configuré (Let's Encrypt via Caddy)
- [ ] `JWT_SECRET` aléatoire et fort (32+ bytes)
- [ ] Postgres password fort, non par défaut
- [ ] MinIO credentials non par défaut
- [ ] Firewall : seul 80/443 exposés
- [ ] OS à jour, security updates auto
- [ ] Backups testés (restore essayé !)

### Hardening
- Containers en read-only filesystem
- User non-root
- Capabilities Linux minimum
- AppArmor / SELinux profile (v1.x)

## Mise à l'échelle

### Vertical
Augmenter CPU/RAM de l'instance serveur BSE.

### Horizontal (v1.x)
- Plusieurs instances BSE avec sticky session par room
- Lookup `room_id → instance_id` dans Redis
- Postgres HA avec replicas read-only

## Costs (self-host)

| Setup | Coût mensuel |
|---|---|
| VPS basique (1 vCPU, 2 GB) pour <20 users | 5-10 € |
| VPS solide (4 vCPU, 8 GB) pour <100 users | 20-40 € |
| Multi-instance HA pour 500+ users | 100+ € |

## Documentation utilisateur

À écrire en docs/user/ :
- Guide d'installation Docker step-by-step
- Configuration OIDC pour chaque provider
- Troubleshooting commun
- FAQ

## Tests d'installation

- Docker compose up fresh : utilisable en <5 min
- Migration upgrade : versions N-1 → N sans perte
- Backup + restore : projet identique après restore

## Liens

- Architecture → [../03-ARCHITECTURE/](../03-ARCHITECTURE/)
- Distribution binaires → [04-distribution-binaires.md](./04-distribution-binaires.md)
- Sécurité → [../09-SECURITE/](../09-SECURITE/)
