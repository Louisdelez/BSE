//! Connection state visible to the rest of the app (status bar, etc.).
//!
//! The enum below is part of the **stable public API** of this crate :
//! the `bse-ui` status bar matches on its variants and calls
//! [`ConnectionState::label`]. Adding variants is allowed, removing or
//! renaming existing ones is **not**.

/// Coarse-grained connection state, displayed to the user.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConnectionState {
    /// No server configured.
    #[default]
    Offline,
    /// Currently attempting to connect.
    Connecting,
    /// Connected and synchronizing.
    Connected,
    /// Lost connection, retrying with exponential backoff.
    Reconnecting,
}

impl ConnectionState {
    /// Short human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Offline => "Offline",
            Self::Connecting => "Connecting...",
            Self::Connected => "Connected",
            Self::Reconnecting => "Reconnecting...",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_offline() {
        assert_eq!(ConnectionState::default(), ConnectionState::Offline);
    }

    #[test]
    fn labels_are_stable() {
        // The bse-ui status bar embeds these strings — any change here is
        // observable in the UI snapshot tests of that crate.
        assert_eq!(ConnectionState::Offline.label(), "Offline");
        assert_eq!(ConnectionState::Connecting.label(), "Connecting...");
        assert_eq!(ConnectionState::Connected.label(), "Connected");
        assert_eq!(ConnectionState::Reconnecting.label(), "Reconnecting...");
    }
}
