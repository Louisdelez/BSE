# 09.02 — Permissions et RBAC

> Qui peut faire quoi sur quel projet.

## Modèle

BSE utilise **RBAC** (Role-Based Access Control) au niveau **projet**.

```
User --(role)--> Project --has--> Resources (canvas, settings, members, ...)
```

## Rôles (v1.0)

| Rôle | Description |
|---|---|
| **Owner** | Créateur du projet. Droits totaux dont suppression. |
| **Facilitator** | Édition + outils de facilitation (timer, mode privé, voting). |
| **Editor** | Édition complète du contenu, pas des settings projet. |
| **Viewer** | Lecture seule, peut commenter. |
| **Commenter** | Lecture seule + commentaires uniquement. |

## Permissions par rôle

| Action | Owner | Facil. | Editor | Viewer | Commenter |
|---|---|---|---|---|---|
| Voir le canvas | ✅ | ✅ | ✅ | ✅ | ✅ |
| Modifier le canvas | ✅ | ✅ | ✅ | ❌ | ❌ |
| Ajouter commentaires | ✅ | ✅ | ✅ | ❌ | ✅ |
| Inviter des membres | ✅ | ⚠️ | ❌ | ❌ | ❌ |
| Retirer des membres | ✅ | ❌ | ❌ | ❌ | ❌ |
| Promouvoir/rétrograder | ✅ | ❌ | ❌ | ❌ | ❌ |
| Activer mode facilitation | ✅ | ✅ | ❌ | ❌ | ❌ |
| Modifier settings projet | ✅ | ❌ | ❌ | ❌ | ❌ |
| Supprimer le projet | ✅ | ❌ | ❌ | ❌ | ❌ |
| Export | ✅ | ✅ | ✅ | ✅ | ✅ |
| Voir l'historique | ✅ | ✅ | ✅ | ✅ | ❌ |

⚠️ = peut inviter mais selon settings projet (toggle « members can invite »).

## Niveaux de partage

### Par invitation explicite
- Owner ajoute par email ou user_id
- Le destinataire reçoit notification

### Par lien (v1.x)
- Lien d'invitation avec rôle pré-défini
- Optionnellement valable N usages ou jusqu'à date X
- E.g., `https://bse.example.com/invite/abc123def`

### Public (v1.x)
- Le projet est accessible sans compte avec rôle minimum (viewer par défaut)
- Configurable : « public link → role »

## Stockage

### Postgres
```sql
CREATE TABLE project_members (
    project_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role TEXT NOT NULL,
    invited_by UUID,
    joined_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (project_id, user_id)
);
```

## Enforcement

### Côté serveur (autoritaire)
À chaque opération :
1. Identifier l'utilisateur (JWT)
2. Récupérer son rôle sur le projet
3. Vérifier que le rôle autorise l'opération
4. Sinon : 403 Forbidden

### Côté client (UX)
- Bouton désactivé si pas permis
- Tooltip : « Vous n'avez pas les droits »
- Mais **jamais** se reposer sur le client pour la sécurité

## Cas runtime

### Rôle change pendant session
Cf [../05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md](../05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md) cas 14.

- Le serveur révoque les futures ops non autorisées
- Le client reçoit `Error::PermissionRevoked`
- UI : notification, basculement en read-only

### Owner quitte
- Doit transférer la propriété avant
- Ou : le projet devient orphelin → archive

## Vérifications atomiques

Toutes les vérifications côté serveur sont **dans la transaction** :

```rust
async fn delete_element(
    user: User,
    project_id: ProjectId,
    element_id: ElementId,
    pool: &Pool,
) -> Result<()> {
    let mut tx = pool.begin().await?;
    
    let role = get_user_role(&mut tx, user.id, project_id).await?;
    if !role.can_edit() {
        return Err(Error::Forbidden);
    }
    
    // ... do delete ...
    tx.commit().await?;
    Ok(())
}
```

## Audit logs

Pour les actions sensibles, une trace est gardée :

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    project_id UUID,
    user_id UUID,
    action TEXT,           -- "project.created", "member.added", "role.changed"
    target_id UUID,         -- ressource ciblée
    metadata JSONB,
    created_at TIMESTAMPTZ
);
```

Visible dans Settings > Audit logs (par owner).

## Limites v1.0

- Pas de permissions au niveau **éléments individuels** (un user édite tout ou rien)
- Pas de teams / organizations (en v1.x)
- Pas de SCIM pour la synchro user d'enterprise (en v2)

## Locks applicatifs (v1.x)

Au-delà du RBAC, on peut **verrouiller un élément** :
- Un éditeur lock un élément le temps de l'éditer
- Les autres ne peuvent pas le modifier (mais peuvent voir)
- Auto-unlock après timeout 5 min ou disconnect

```rust
pub struct ElementLock {
    pub element_id: ElementId,
    pub locked_by: UserId,
    pub locked_until: DateTime,
}
```

## Privacy par défaut

- Un nouveau projet est **privé** au créateur
- Doit explicitement partager pour donner accès
- Pas de "discover" public sauf opt-in

## Tests

- Owner peut tout, Editor peut éditer mais pas inviter, Viewer ne peut rien modifier
- Tentative d'op non autorisée → 403
- Audit log enregistré pour les actions sensibles
- Locks fonctionnent correctement

## Liens

- Auth → [01-authentification.md](./01-authentification.md)
- Threat model → [04-modele-de-menace.md](./04-modele-de-menace.md)
