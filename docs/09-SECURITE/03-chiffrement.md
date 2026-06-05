# 09.03 — Chiffrement

> TLS standard pour la transport, E2EE optionnelle pour la confidentialité maximale.

## Niveaux de chiffrement

### Niveau 1 — TLS (transport)
**Toujours actif** en production.
- Toutes les requests HTTP/WS sont en HTTPS/WSS
- TLS 1.3 (TLS 1.2 minimum)
- Implementation : `rustls` (préféré OpenSSL)

### Niveau 2 — Chiffrement au repos (storage)
**Recommandé**.
- Disques chiffrés (LUKS, BitLocker, FileVault) au niveau OS
- S3/MinIO encryption-at-rest (SSE)
- Postgres TDE optionnel
- BSE n'ajoute pas une couche supplémentaire — délègue à l'infra

### Niveau 3 — End-to-end (E2EE) — optionnel
**Activable par projet**.
- Le serveur ne peut **pas lire** le contenu du projet
- Inspiré d'Excalidraw

## TLS

### Certificat
- **Let's Encrypt** via reverse proxy (Caddy, Nginx)
- Auto-renouvellement
- Pas de cert dans BSE directement → délégué au proxy

### Configuration rustls
```rust
let cert_chain = load_certs(&config.tls_cert)?;
let private_key = load_private_key(&config.tls_key)?;

let server_config = rustls::ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(cert_chain, private_key)?;
```

### HSTS
- Header `Strict-Transport-Security: max-age=31536000`
- Force HTTPS sur tous les browsers

### CSP
Pas applicable (app native), mais pour le admin panel web futur, CSP strict.

## Chiffrement E2E (option par projet)

### Vision
Quand un projet est créé en **mode E2EE** :
- Une **room key** AES-256 est générée côté client à la création
- La clé n'est **jamais** envoyée au serveur
- Tous les éléments CRDT sont chiffrés avec cette clé avant envoi
- Le serveur ne voit que du chiffrement, il relaie sans pouvoir lire

### Inspiration : Excalidraw
Excalidraw fait ça depuis 2020, prouvant la faisabilité.

### Partage de la clé
À l'invitation d'un nouveau peer :
- La clé doit être partagée via canal sécurisé
- **Option A** : QR code généré sur le device d'un peer authentifié, scanné par le nouveau
- **Option B** : URL d'invitation contenant la clé dans le fragment (`#`) — jamais envoyée HTTP
- **Option C** : Signal-like protocol avec key exchange (X3DH) — surdimensionné

**v1.0** : option B (URL fragment), comme Excalidraw.

### Implémentation
```rust
// À la création
let room_key = generate_aes256_key();
let url = format!("https://bse.example.com/p/{}#{}", project_id, base64(room_key));
// → user partage cette URL

// À l'envoi d'une op
let plaintext = encode_op(&op);
let nonce = generate_nonce();
let ciphertext = aes_gcm_encrypt(&room_key, &nonce, &plaintext);
ws.send(WsMessage::Binary([nonce, ciphertext].concat())).await;

// À la réception
let (nonce, ciphertext) = split_message(&data);
let plaintext = aes_gcm_decrypt(&room_key, &nonce, &ciphertext)?;
let op = decode_op(&plaintext)?;
```

### Crate Rust
- `aes-gcm` (RustCrypto) : AES-GCM authenticated encryption
- `chacha20poly1305` : alternative ChaCha20-Poly1305 (plus rapide sur CPU sans AES-NI)

### Snapshots côté serveur
En mode E2EE, le serveur **stocke aussi chiffré**. Les snapshots S3 sont chiffrés avec la room key — donc personne d'autre que les peers ne peut lire.

### Trade-offs E2EE
- ✅ Confidentialité totale (vis-à-vis du serveur)
- ❌ Pas de recherche server-side
- ❌ Pas d'IA server-side
- ❌ Si la room key est perdue : projet **irrécupérable**
- ❌ Pas de récupération admin

E2EE est pour les cas où **la confidentialité prime sur la commodité**.

### UX E2EE
- À la création : checkbox « Activer le chiffrement bout-en-bout »
- Indicateur visible dans l'UI quand actif (icône cadenas)
- Avertissement à l'invitation : « partagez le lien via canal sécurisé »

## Chiffrement des passwords

### Au repos
Argon2id (cf [01-authentification.md](./01-authentification.md))

### En transit
HTTPS → jamais en clair sur le réseau

### Jamais en log
Filtrer le password dans tous les logs (config tracing).

## Chiffrement des JWT

### Signature
RS256 (RSA-SHA256) avec clé asymétrique. Le secret de signature n'est jamais transmis aux clients.

### Pas de chiffrement du contenu
Les JWT BSE sont **signés** mais pas chiffrés. Le contenu (email, user_id) est lisible par n'importe qui ayant le token. C'est intentionnel et standard.

Pour des info ultra-sensibles → ne pas les mettre dans le JWT.

## Génération de aléatoire sécurisé

Toujours utiliser `OsRng` (cryptographically secure) pour :
- Génération de clés
- Tokens d'invitation
- Nonces

```rust
use rand::rngs::OsRng;
use rand::Rng;

let mut key = [0u8; 32];
OsRng.fill(&mut key);
```

Crate `ring` ou `rustcrypto` selon préférence.

## Audit cryptographique

Pour la v1.0+ critique :
- Revue par un cryptographe externe
- Bug bounty
- Documentation des choix
- Pas de crypto maison — uniquement des primitives standard

## Tests

- TLS handshake : OK avec cert valide, fail avec self-signed sans config
- E2EE : 2 clients avec même room key communiquent ; client sans clé ne peut pas
- Réception d'un message chiffré altéré : décrypt fail (GCM authentifié)
- Génération de clés : pas de répétition

## Threats couvertes

- Eavesdropping réseau : TLS prévient
- Eavesdropping côté serveur (admin curieux) : E2EE prévient (en mode E2E)
- Vol de DB : encryption at rest + E2EE prévient
- MITM : TLS + cert pinning (v1.x)

## Threats hors scope BSE

- Compromise de l'OS client (keylogger, etc.) : pas notre problème
- Compromise du device : pareil
- Côté quantique : v2+ (CRYSTALS-Kyber etc.)

## Liens

- Auth → [01-authentification.md](./01-authentification.md)
- Threat model → [04-modele-de-menace.md](./04-modele-de-menace.md)
- Excalidraw E2EE → [../02-ETAT-DE-LART/03-excalidraw.md](../02-ETAT-DE-LART/03-excalidraw.md)
