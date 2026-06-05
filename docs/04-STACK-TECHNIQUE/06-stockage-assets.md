# 04.06 — Stockage des assets (images, fichiers)

> Comment BSE gère les binaires : upload, dé-duplication, livraison, cache.

## Le problème

Un canvas peut contenir des **images**, **documents importés**, **exports**, etc. Ces fichiers :
- Pèsent typiquement 100 KB - 10 MB
- Sont parfois utilisés dans plusieurs projets
- Doivent être servis rapidement aux peers connectés
- Ne doivent pas tout encombrer du WAL CRDT

## Solution : système d'assets séparé

Les assets sont stockés **hors du CRDT**. Le CRDT contient seulement la **référence** (sha256 + metadata légère). L'asset binaire vit en S3.

```
CRDT (Postgres + clients)
  └── Element { kind: Image { asset_id: "sha256:abc123...", width, height } }

S3 / MinIO
  └── /projects/{pid}/assets/abc123...  (le binaire)
```

## Le hash de contenu (content-addressed)

Chaque asset est identifié par son **SHA-256**. Avantages :
- ✅ **Dé-duplication automatique** : 10 utilisateurs uploadent la même image → 1 seul stockage
- ✅ **Intégrité** : le client peut vérifier le hash après download
- ✅ **Cache infini** : un asset avec un sha256 donné est immuable

## Cycle de vie d'un upload

```
[Alice] drag & drop "logo.png" sur la toile

  1. Client calcule SHA-256 du fichier
  2. Client interroge le serveur :
     GET /api/assets/exists/{sha256}
     ─► 200 si déjà uploadé, 404 sinon
     
  3a. Si exists → skip upload
  3b. Si non → POST /api/projects/{pid}/assets
       Body: multipart (file)
       ─► Server stocke en S3 sous /projects/{pid}/assets/{sha256}
       ─► Retourne metadata (size, mime, width, height)
       
  4. Client crée l'Element { kind: Image { asset_id, width, height } }
  5. Op CRDT broadcast vers serveur et autres peers
  
  6. [Bob] reçoit l'op
  7. [Bob] charge l'asset si pas en cache :
     GET /api/projects/{pid}/assets/{sha256}
     ─► 200 + binaire (ou redirect signé S3)
     
  8. [Bob] décode et affiche l'image
```

## Contraintes et limites

### Taille maximale par asset
- v1.0 : **20 MB** par fichier
- Configurable par instance
- Au-delà : refusé avec error message clair

### Types acceptés
- **Images** : PNG, JPG, GIF, WEBP, SVG (v1)
- **Documents** : PDF (v1.x, affichage embed)
- **Vidéos** : MP4 (v2 peut-être)

Validation MIME + magic bytes (anti-spoof).

### Anti-abus
- Rate limit upload : 10 fichiers / minute / utilisateur
- Quota par projet : 1 GB en v1.0
- Total quota par compte : 5 GB (self-host = pas de limite)

## Pipeline d'image côté serveur

À l'upload, le serveur :
1. Sauve l'original en S3 sous `{sha256}/original.{ext}`
2. Génère 2-3 variantes via `image` crate :
   - `{sha256}/thumb.webp` (200x200 max, ~10 KB)
   - `{sha256}/medium.webp` (1024x1024 max, ~100 KB)
3. Stocke metadata Postgres :
   ```sql
   CREATE TABLE assets (
       sha256 TEXT PRIMARY KEY,
       size BIGINT,
       mime_type TEXT,
       width INTEGER,
       height INTEGER,
       has_thumb BOOLEAN,
       has_medium BOOLEAN,
       uploaded_at TIMESTAMPTZ
   );
   ```

### Lib Rust pour image
- `image` : décodage / encodage
- `webp` : encodage WebP
- `imageproc` : opérations (resize, etc.)

## Livraison aux clients

### Mode auto-hébergé simple
- Le serveur BSE proxy les fichiers depuis S3/MinIO
- Routes `/api/projects/{pid}/assets/{sha256}` retournent le binaire
- Headers de cache aggressifs : `Cache-Control: public, max-age=31536000, immutable`

### Mode production (CDN)
- Le serveur retourne une **URL signée** S3/R2/CloudFront
- TTL court (1 h) sur la signature
- Client télécharge directement du CDN
- Économie de bande passante serveur

## Cache côté client

### En mémoire
- LRU `HashMap<Sha256, Arc<DecodedImage>>` 
- Capacité : 256 MB max
- Eviction quand limite atteinte

### Sur disque
- `assets_cache` table SQLite + fichiers dans `cache/` dossier user
- Capacité : 1 GB max
- LRU avec `last_used`

### Pre-fetch
À l'ouverture d'un projet, le client peut pre-charger les assets visibles dans le viewport initial.

## Optimisations

### Variant selection
Selon le zoom :
- Zoom out → variante `thumb` ou `medium`
- Zoom in → variante `original` (haute résolution)
- Crossfade lors du switch

### Format préféré
- WebP en priorité (taille réduite vs PNG/JPG)
- AVIF en v2 (encore meilleur, encore mal supporté côté décodage Rust)

### Lazy loading
- Ne charger que les assets dont l'element est dans le viewport
- Annuler le download si l'utilisateur scroll vite vers une autre zone

### Pre-warming
À l'ouverture, on précharge les **N premiers assets référencés** dans la scène pour éviter le flash.

## Exports (assets de sortie)

Exports PDF/PNG/SVG sont aussi stockés temporairement en S3 :
```
/projects/{pid}/exports/{export_id}/...
```

TTL : 24 h (cleanup automatique).

## Stockage local sans serveur

En mode **standalone** (pas de serveur configuré), les assets sont stockés purement sur disque :

```
~/.local/share/bse/projects/{pid}/assets/{sha256}
```

Pas de dé-duplication cross-projet (limite acceptable).

## Sécurité

### Sécurité des fichiers uploadés
- Validation magic bytes (anti-MIME spoofing)
- Pas d'exécution côté serveur
- Sanitization des SVG (anti-XSS — script tags retirés)
- Scan antivirus optionnel (ClamAV intégration possible)

### Sécurité d'accès
- Vérification droits sur le projet avant download
- URLs signées avec TTL court en mode CDN
- Pas de listing public du bucket

## Décisions

| Décision | Choix | Raison |
|---|---|---|
| Adressage | SHA-256 content-addressed | Dé-dup native |
| Backend | S3-compatible (MinIO self-host) | Standard, mature |
| Variants | thumb + medium + original | Bande passante |
| Cache client | RAM + disque LRU | Performance |
| CDN | Optionnel, prod | Économies BW |
| Lib image | `image` crate | Standard Rust |
