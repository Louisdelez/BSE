# BSE deployment guide

This document covers self-hosting `bse-server` for production
collaboration. The reference setup runs the server in Docker behind
Caddy for automatic TLS — but every piece is swappable.

## What you are deploying

`bse-server` is a single Rust binary :

- HTTP + WebSocket server on a single port (default `8080`).
- SQLite database holding users, room memberships and CRDT snapshots
  (default path `./data/server.sqlite`, configurable via
  `BSE_SERVER_DATA_DIR`).
- No background workers, no message broker, no Redis. The desktop
  client connects directly over WebSocket.

The desktop client (`bse-app`) is shipped as a native binary per OS
(see "Release binaries" below).

## Minimum requirements

- Linux amd64 host (the Dockerfile is multi-stage Debian Bookworm).
- 1 vCPU, 512 MB RAM, 1 GB disk for the SQLite store at a small
  team's scale. Scale up the disk if you expect dozens of
  multi-MB boards.
- Ports `80` + `443` reachable from the internet for the Let's
  Encrypt HTTP-01 challenge (only the first time and at renewal).
- A domain name pointing to the host (e.g. `bse.example.com`).

## Quick start with docker-compose

```bash
git clone https://github.com/Louisdelez/BSE.git
cd BSE

# 1. Copy the env template and fill it in.
cp .env.example .env
# Edit .env :
#   BSE_JWT_SECRET   - generate with `openssl rand -hex 48`
#   BSE_ALLOWED_ORIGINS - leave empty unless you serve a web client

# 2. Point your domain at this host, then edit the Caddyfile :
#   replace `bse.example.com` with your actual domain.

# 3. Build + run.
docker compose up -d --build

# 4. Verify.
curl https://bse.example.com/health
# → {"status":"ok"}

curl https://bse.example.com/ready
# → {"status":"ready","db_ok":true,"uptime_secs":42, ...}
```

The first request triggers Caddy's ACME flow ; certificates are
cached in the `caddy-data` volume across restarts.

## Environment variables

| Variable | Default | Purpose |
|---|---|---|
| `BSE_BIND_ADDR` | `0.0.0.0:8080` | Listen socket. |
| `BSE_SERVER_DATA_DIR` | `./data` | Directory for `server.sqlite`. |
| `BSE_JWT_SECRET` | *(process-local random)* | HS256 secret. **Required in production** — without it, restarting the server invalidates all sessions. |
| `BSE_REQUIRE_AUTH` | *(unset = off)* | Set to `1` to require a valid JWT on WS upgrade. **Required in production.** |
| `BSE_ALLOWED_ORIGINS` | *(unset = permissive)* | Comma-separated list of CORS origins. Set when serving a web client. |
| `RUST_LOG` | *(unset)* | Standard `tracing` env-filter (e.g. `info,bse=debug`). |

## Persistence

- SQLite is configured with WAL journaling and `synchronous=NORMAL`
  on first open. Data is durable across process restarts and
  crashes.
- Back up the data directory by copying `server.sqlite` (+ the
  matching `-wal` / `-shm` sidecars if present) while the server
  is stopped, or by using `sqlite3 server.sqlite ".backup"` while
  running.
- Restoring is a file copy back to the same path.

## Sign-in flow recap

1. Client `POST /api/auth/register` to create an account, then
   `POST /api/auth/login` for the JWT pair.
2. Client passes the access token as `?token=...` on the WS URL.
3. Server verifies the token and checks room membership.
4. Client periodically `POST /api/auth/refresh`es to keep its
   access token fresh.

The demo seed account `demo@bse.app` / `demo1234` is created the
**first time** the server starts on an empty database — disable it
in production by registering a real user and then deleting the
demo row, or by setting `BSE_REQUIRE_AUTH=1` and never sharing
those creds.

## Updates and migrations

The SQLite schema is versioned through a `schema_migrations` table.
On every boot the server applies any pending migration in a single
transaction. Pull a newer image, restart the container — the data
volume is preserved.

## Monitoring

- `/health` is a liveness probe (cheap, no DB).
- `/ready` is a readiness probe that pings the DB and reports
  uptime. Returns `503` when SQLite is unreachable so a load
  balancer can drain.
- `RUST_LOG=info` emits structured JSON-friendly logs (one event
  per line). Pipe them to your aggregator of choice.

## Release binaries

Each `v0XX` tag triggers a GitHub Actions workflow
(`.github/workflows/release.yml`) that builds `bse-app` and
`bse-server` for Linux, macOS and Windows and attaches the
archives to the GitHub Release. The desktop binaries are
*not* signed/notarized yet — that lands when v1.0 is on the table.

## Hardening checklist

- [x] HTTPS at the edge (Caddy or nginx).
- [x] `BSE_JWT_SECRET` is a long random value.
- [x] `BSE_REQUIRE_AUTH=1`.
- [x] `BSE_ALLOWED_ORIGINS` is set if a web client exists.
- [x] Rate limits on `/api/auth/*` (`tower_governor`, baked in).
- [x] Body limit (1 MiB JSON / WS frame, baked in).
- [x] Periodic backup of the SQLite data dir.
- [ ] Off-host log shipping (Loki / Cloudwatch / Datadog).
- [ ] Brute-force lockout policy (planned post-v025).
