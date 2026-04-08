//! Session management with auto-reconnection.

use crate::browser::{BrowserConnector, BrowserError, BrowserSession};

/// Manages the lifecycle of a browser session, including automatic
/// reconnection when the connection drops.
pub struct SessionManager {
    /// The connector used to establish sessions.
    pub connector: BrowserConnector,
    /// The currently active session, if any.
    pub active_session: Option<BrowserSession>,
    /// Whether to automatically attempt reconnection on failure.
    pub auto_reconnect: bool,
    /// Delay between reconnection attempts, in milliseconds.
    pub reconnect_interval_ms: u64,
}

impl SessionManager {
    /// Create a new session manager for the given debug port.
    ///
    /// Auto-reconnect is enabled by default with a 2-second interval.
    pub fn new(debug_port: u16) -> Self {
        Self {
            connector: BrowserConnector::new(debug_port),
            active_session: None,
            auto_reconnect: true,
            reconnect_interval_ms: 2000,
        }
    }

    /// Return a reference to the active session, connecting first if needed.
    ///
    /// If `auto_reconnect` is enabled and the current session is disconnected,
    /// this will attempt to re-establish the connection.
    pub async fn ensure_connected(&mut self) -> Result<&BrowserSession, BrowserError> {
        let needs_connect = match self.active_session {
            Some(ref s) => !s.connected,
            None => true,
        };

        if needs_connect {
            let session = self.connector.connect().await?;
            self.active_session = Some(session);
        }

        Ok(self.active_session.as_ref().unwrap())
    }

    /// Disconnect from the browser and drop the active session.
    pub async fn disconnect(&mut self) {
        self.active_session = None;
    }

    /// Returns `true` if there is an active, connected session.
    pub fn is_connected(&self) -> bool {
        self.active_session.as_ref().is_some_and(|s| s.connected)
    }
}
