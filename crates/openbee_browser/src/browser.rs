//! Browser automation abstraction over the Chrome DevTools Protocol (CDP).
//!
//! # Privacy Warning
//!
//! This module connects to the user's **already-running** browser session.
//! It does **not** store, forward, or log any credentials, cookies, or
//! authentication tokens. All operations target `localhost` only.

use serde::{Deserialize, Serialize};

/// Connects to a running Chrome/Chromium/Edge browser via the Chrome DevTools
/// Protocol on a local TCP debug port.
///
/// The user must start their browser with remote debugging enabled:
/// ```text
/// chrome --remote-debugging-port=9222
/// ```
pub struct BrowserConnector {
    /// The local TCP port where the browser exposes CDP (typically 9222).
    pub debug_port: u16,
    /// Connection timeout in milliseconds.
    pub timeout_ms: u64,
}

/// Metadata for a single browser tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTab {
    /// Page title.
    pub title: String,
    /// Current URL.
    pub url: String,
    /// CDP target identifier.
    pub tab_id: String,
}

/// Captured content from a browser page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    /// URL of the page.
    pub url: String,
    /// Document title.
    pub title: String,
    /// Raw HTML source.
    pub html: String,
    /// Extracted visible text (tags stripped).
    pub text: String,
    /// Unix timestamp when the content was captured.
    pub timestamp: u64,
}

impl BrowserConnector {
    /// Create a new connector targeting the given debug port.
    pub fn new(debug_port: u16) -> Self {
        Self {
            debug_port,
            timeout_ms: 5000,
        }
    }

    /// Attempt to connect to the running browser instance.
    ///
    /// A real implementation would:
    /// 1. HTTP GET `http://localhost:{port}/json/version` to verify the browser
    /// 2. Open a WebSocket to the `webSocketDebuggerUrl` from the response
    ///
    /// This placeholder returns a descriptive error because no HTTP client
    /// crate is available.
    pub async fn connect(&self) -> Result<BrowserSession, BrowserError> {
        // Placeholder: in production this would open a WebSocket connection to
        // the CDP endpoint at ws://localhost:{port}/devtools/browser/{id}.
        Err(BrowserError::ConnectionError(format!(
            "Placeholder: would connect to CDP on localhost:{}. \
             A real implementation requires a WebSocket client to communicate \
             with the browser's DevTools protocol endpoint.",
            self.debug_port
        )))
    }

    /// List all open tabs by querying `http://localhost:{port}/json/list`.
    ///
    /// This is a placeholder — a real implementation would perform an HTTP GET
    /// and deserialize the JSON array of targets.
    pub async fn list_tabs(&self) -> Result<Vec<BrowserTab>, BrowserError> {
        Err(BrowserError::ConnectionError(format!(
            "Placeholder: would GET http://localhost:{}/json/list to enumerate tabs. \
             Requires an HTTP client to fetch the target list from CDP.",
            self.debug_port
        )))
    }
}

/// An active connection to a browser instance.
pub struct BrowserSession {
    /// The debug port this session is connected to.
    pub debug_port: u16,
    /// Whether the WebSocket connection is alive.
    pub connected: bool,
}

impl BrowserSession {
    /// Navigate a tab to the given URL.
    ///
    /// CDP method: `Page.navigate`
    pub async fn navigate(&self, tab_id: &str, url: &str) -> Result<(), BrowserError> {
        if !self.connected {
            return Err(BrowserError::NotConnected);
        }
        Err(BrowserError::NavigationFailed(format!(
            "Placeholder: would send CDP Page.navigate(url={}) to tab {}. \
             Requires an active WebSocket connection to the browser.",
            url, tab_id
        )))
    }

    /// Retrieve the full page content (HTML + extracted text) for a tab.
    ///
    /// CDP methods: `Runtime.evaluate` with `document.title`,
    /// `document.documentElement.outerHTML`, and inner-text extraction.
    pub async fn get_page_content(&self, tab_id: &str) -> Result<PageContent, BrowserError> {
        if !self.connected {
            return Err(BrowserError::NotConnected);
        }
        Err(BrowserError::ConnectionError(format!(
            "Placeholder: would evaluate JS in tab {} to capture DOM content. \
             Requires CDP Runtime.evaluate over WebSocket.",
            tab_id
        )))
    }

    /// Execute arbitrary JavaScript in a tab and return the string result.
    ///
    /// CDP method: `Runtime.evaluate`
    ///
    /// # Safety Note
    ///
    /// This is provided for **read-only** inspection (e.g. extracting text).
    /// Callers should avoid injecting scripts that modify page state.
    pub async fn evaluate_js(&self, tab_id: &str, script: &str) -> Result<String, BrowserError> {
        if !self.connected {
            return Err(BrowserError::NotConnected);
        }
        Err(BrowserError::JsError(format!(
            "Placeholder: would run `{}` in tab {} via CDP Runtime.evaluate.",
            script, tab_id
        )))
    }

    /// Wait until an element matching `selector` appears in the DOM, or
    /// until `timeout_ms` elapses.
    ///
    /// Polls via `Runtime.evaluate` with `document.querySelector(selector)`.
    pub async fn wait_for_selector(
        &self,
        tab_id: &str,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<bool, BrowserError> {
        if !self.connected {
            return Err(BrowserError::NotConnected);
        }
        Err(BrowserError::Timeout(format!(
            "Placeholder: would poll for selector '{}' in tab {} for up to {}ms.",
            selector, tab_id, timeout_ms
        )))
    }

    /// Get the `innerText` of the first element matching `selector`.
    pub async fn get_element_text(
        &self,
        tab_id: &str,
        selector: &str,
    ) -> Result<String, BrowserError> {
        if !self.connected {
            return Err(BrowserError::NotConnected);
        }
        Err(BrowserError::JsError(format!(
            "Placeholder: would evaluate \
             `document.querySelector('{}').innerText` in tab {}.",
            selector, tab_id
        )))
    }

    /// Type text into an input element matching `selector`.
    ///
    /// CDP methods: `DOM.querySelector` + `Input.dispatchKeyEvent` per character.
    pub async fn type_text(
        &self,
        tab_id: &str,
        selector: &str,
        text: &str,
    ) -> Result<(), BrowserError> {
        if !self.connected {
            return Err(BrowserError::NotConnected);
        }
        Err(BrowserError::JsError(format!(
            "Placeholder: would focus '{}' in tab {} and dispatch key events for '{}'.",
            selector, tab_id, text
        )))
    }

    /// Click the first element matching `selector`.
    ///
    /// CDP methods: `DOM.querySelector` + `Input.dispatchMouseEvent`.
    pub async fn click(&self, tab_id: &str, selector: &str) -> Result<(), BrowserError> {
        if !self.connected {
            return Err(BrowserError::NotConnected);
        }
        Err(BrowserError::JsError(format!(
            "Placeholder: would click '{}' in tab {}.",
            selector, tab_id
        )))
    }
}

/// Errors from browser automation operations.
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error(
        "Browser not running or debug port not open. \
         Start Chrome with: chrome --remote-debugging-port=9222"
    )]
    NotConnected,

    #[error("Tab not found: {0}")]
    TabNotFound(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("JavaScript error: {0}")]
    JsError(String),

    #[error("Timeout waiting for selector: {0}")]
    Timeout(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}
