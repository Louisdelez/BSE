# 09.01 — Authentification

> Comment un utilisateur prouve son identité à BSE.

## Modes d'authentification supportés

### 1. Email + password local (built-in)
- Inscription via email + mot de passe
- Argon2id pour le hash
- Email verification optionnelle
- Pour : auto-hébergement basique sans IdP externe

### 2. OpenID Connect (OIDC)
- Google, GitHub, Microsoft, Keycloak, Authentik, Authelia…
- Standard moderne
- Pour : entreprises avec IdP existant

### 3. Magic link (v1.x)
- Email un lien, click → connecté
- Pas de password à gérer
- Pour : utilisateurs occasionnels

### 4. Pas d'auth (mode local-only)
- Pour mode standalone sans serveur
- Aucune notion d'identité

## Flow OIDC (recommandé)

### Setup
- Admin du serveur BSE configure :
  - Issuer URL (ex: `https://accounts.google.com`)
  - Client ID
  - Client Secret
  - Redirect URI (`bse://auth/callback` pour app desktop)

### Flow utilisateur (desktop)

```
1. User click "Login with Google"
2. BSE app ouvre browser système :
   https://accounts.google.com/o/oauth2/v2/auth?
     client_id=...&redirect_uri=bse://auth/callback&
     scope=openid+profile+email&...
3. User s'authentifie chez Google
4. Google redirige vers bse://auth/callback?code=AUTHCODE
5. BSE app intercepte le protocole bse:// (registered scheme)
6. BSE app POST vers son backend : /api/auth/oidc/exchange
   { code: AUTHCODE }
7. Backend BSE échange code contre tokens auprès de Google
8. Backend BSE crée / met à jour l'user, retourne JWT
9. BSE app stocke JWT
10. Désormais, toutes les requests incluent Authorization: Bearer <jwt>
```

### Custom URL scheme
- macOS : registered via Info.plist
- Windows : registered via registry
- Linux : via desktop file

### Alternative : loopback HTTP
Pour les cas où le custom scheme pose problème :
- BSE app démarre un serveur HTTP local sur port aléatoire (e.g., `http://localhost:43217`)
- Redirect URI = `http://localhost:43217/callback`
- Captures le code et ferme le serveur

C'est le pattern recommandé par OAuth 2.1 pour les apps natives.

## JWT (JSON Web Tokens)

### Structure
```json
{
  "header": { "alg": "RS256", "typ": "JWT" },
  "payload": {
    "iss": "https://bse.example.com",
    "sub": "user_uuid_xxx",
    "aud": "bse-client",
    "exp": 1735689600,
    "iat": 1735603200,
    "email": "alice@example.com",
    "name": "Alice"
  },
  "signature": "..."
}
```

### Signature
- **RS256** (RSA-SHA256) : asymétrique
- Le serveur signe avec une clé privée
- Les clients (et autres services) vérifient avec la clé publique
- Permet de distribuer la vérification sans partager le secret

### Durée de vie
- Access token : 15 min
- Refresh token : 30 jours
- Refresh : nouveau access token sans réauth

### Stockage côté client
- Access token : RAM uniquement
- Refresh token : OS keyring (`keyring` crate) — secure storage
- Pas dans des fichiers texte sur disque

## Password local (mode built-in)

### Hash
- **Argon2id** (recommandation OWASP 2024)
- Paramètres : m=64 MB, t=3, p=1
- Crate Rust : `argon2`

### Stockage
- Table `users` : `password_hash` (incluant salt + paramètres)
- Jamais en clair, jamais loggué

### Reset
- Email de reset (v1.x)
- Token signé valable 1 h

### Verrouillage
- Après 5 tentatives échouées : rate limit 1 min
- Puis 5 min, 1 h, etc.

## API endpoints

```
POST /api/auth/register        { email, password } → user
POST /api/auth/login            { email, password } → { access_token, refresh_token }
POST /api/auth/refresh          { refresh_token } → { access_token }
POST /api/auth/logout           Authorization required
GET  /api/auth/oidc/start       → redirect URL (PKCE state in session)
POST /api/auth/oidc/exchange    { code } → { tokens }
GET  /api/me                    Authorization required → user info
```

## WebSocket authentication

Le JWT est passé au moment de la connexion WS :
- Soit en query param : `wss://srv/ws/rooms/123?token=eyJ...`
- Soit en subprotocol : `bse.v1.token.eyJ...`

Le serveur vérifie au handshake, refuse si invalide.

## Session management

- 1 user peut avoir N sessions actives (multi-device)
- Liste des sessions visible dans Settings
- Possibilité de révoquer une session particulière

## Refresh token rotation

À chaque utilisation du refresh, le serveur émet un nouveau refresh + invalide l'ancien. Permet de détecter le vol (un attaquant utilisera un refresh révoqué → alarm).

## Multi-factor authentication (v1.x)

- TOTP (Google Authenticator, Authy)
- Recovery codes
- v2 : passkeys (WebAuthn)

## Sécurité spécifique

### CSRF
N/A pour API JSON avec Bearer (pas de cookies SameSite). Mais protection sur tout endpoint accepting cookies (rare).

### Token theft
- Refresh rotation détecte
- Possibilité de "logout all sessions"

### Session fixation
N/A avec JWT stateless.

### Brute force
Rate limit sur `/login` + Argon2 lent par design.

### Email enumeration
Réponse identique pour user existant ou non sur l'API publique.

## Anonyme / public access (v1.x)

Pour des projets publics (cf [02-permissions-rbac.md](./02-permissions-rbac.md)) :
- Token guest généré pour la session
- Limité en permissions (view-only typiquement)

## Lib Rust

- `oauth2` : flow OAuth standard
- `openidconnect` : OIDC sur top d'oauth2
- `jsonwebtoken` : encode/décode JWT
- `argon2` : password hashing
- `keyring` : OS secure storage côté client

## Tests

- Login email/password : succès, fail (mauvais pwd), rate limit
- OIDC flow : succès, refus par IdP
- Refresh : succès, refresh expiré, refresh révoqué (rotation)
- WS auth : token valide, expiré, manquant

## Liens

- Permissions → [02-permissions-rbac.md](./02-permissions-rbac.md)
- Threat model → [04-modele-de-menace.md](./04-modele-de-menace.md)
- Architecture serveur → [../03-ARCHITECTURE/03-serveur.md](../03-ARCHITECTURE/03-serveur.md)
