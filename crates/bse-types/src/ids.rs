//! Strongly-typed identifiers.
//!
//! Each entity in BSE has its own newtype wrapping a [`uuid::Uuid`].
//! This prevents accidentally passing a `ProjectId` where an `ElementId`
//! is expected, which would otherwise compile but be semantically wrong.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generate a strongly-typed UUID newtype with all the boilerplate.
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(
            Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Generate a new random v4 identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Generate a new time-ordered v7 identifier.
            ///
            /// Prefer this over [`Self::new`] when you want identifiers
            /// that sort by creation time (useful for database indexes).
            #[must_use]
            pub fn new_v7() -> Self {
                Self(Uuid::now_v7())
            }

            /// Construct from an existing [`Uuid`].
            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Access the underlying [`Uuid`].
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new_v7()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(Uuid::from_str(s)?))
            }
        }
    };
}

define_id!(
    /// Identifier of a BSE project (the root container).
    ProjectId
);

define_id!(
    /// Identifier of an element on the canvas (shape, pen stroke, text, ...).
    ElementId
);

define_id!(
    /// Identifier of a user account.
    UserId
);

define_id!(
    /// Identifier of a connected peer in a room.
    ///
    /// One user can have several peers (e.g., desktop + mobile).
    PeerId
);

define_id!(
    /// Identifier of an asset (image, file).
    ///
    /// Conceptually a content-addressed hash, but stored as a UUID
    /// for cross-tooling consistency. The actual SHA-256 is held in metadata.
    AssetId
);

define_id!(
    /// Identifier of a comment anchored on the canvas.
    CommentId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_types_are_distinct() {
        // This is mostly a compile-time assertion : different newtypes
        // cannot be assigned to each other.
        let _p = ProjectId::new();
        let _e = ElementId::new();
    }

    #[test]
    fn v7_ids_are_time_ordered() {
        let a = ElementId::new_v7();
        let b = ElementId::new_v7();
        assert!(a.as_uuid() < b.as_uuid() || a.as_uuid() == b.as_uuid());
    }

    #[test]
    fn roundtrip_string() {
        let id = ProjectId::new();
        let s = id.to_string();
        let parsed: ProjectId = s.parse().expect("valid uuid string");
        assert_eq!(id, parsed);
    }

    #[test]
    fn roundtrip_serde_json() {
        let id = UserId::new();
        let json = serde_json::to_string(&id).expect("serialize");
        let back: UserId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id, back);
    }
}
