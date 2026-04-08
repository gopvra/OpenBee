//! Reads AI-generated responses from web-based AI platforms.
//!
//! # How It Works
//!
//! The user logs into an AI service (e.g. a chatbot) in their normal browser,
//! then this reader connects to that browser session and extracts the AI's
//! response text from the DOM using CSS selectors.
//!
//! # Privacy Guarantees
//!
//! - This reader **NEVER** accesses, stores, or transmits user credentials,
//!   cookies, or session tokens.
//! - It reads **only** the visible text content from the page DOM.
//! - All communication is local (`localhost` only).

use serde::{Deserialize, Serialize};

use crate::browser::{BrowserError, BrowserSession};

/// Reads AI responses from a web-based AI platform.
pub struct AiReader {
    /// The browser session to read from.
    pub session: BrowserSession,
    /// Which AI platform's DOM structure to expect.
    pub platform: AiPlatform,
}

/// Identifies the AI platform and provides CSS selectors for reading responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiPlatform {
    /// A user-defined platform with custom CSS selectors.
    Custom {
        /// Human-readable platform name.
        name: String,
        /// CSS selector for the AI response container element.
        response_selector: String,
        /// CSS selector for the prompt input field.
        input_selector: String,
        /// CSS selector for the send/submit button.
        send_selector: String,
    },
}

/// A captured AI response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    /// Name of the platform the response was read from.
    pub platform: String,
    /// The text content of the AI response.
    pub content: String,
    /// Unix timestamp when the response was captured.
    pub timestamp: u64,
    /// Model name if it could be detected from the page, otherwise `None`.
    pub model: Option<String>,
}

impl AiReader {
    /// Create a new reader for the given browser session and AI platform.
    pub fn new(session: BrowserSession, platform: AiPlatform) -> Self {
        Self { session, platform }
    }

    /// Read the latest AI response visible on the current page.
    ///
    /// This extracts the `innerText` of the last element matching the
    /// platform's `response_selector`.
    pub async fn read_latest_response(&self) -> Result<AiResponse, AiReaderError> {
        if !self.session.connected {
            return Err(AiReaderError::Browser(BrowserError::NotConnected));
        }

        let (name, response_selector) = match &self.platform {
            AiPlatform::Custom {
                name,
                response_selector,
                ..
            } => (name.clone(), response_selector.clone()),
        };

        // In a real implementation we would:
        // 1. Call session.evaluate_js() with JS that finds the last element
        //    matching `response_selector` and returns its innerText.
        // 2. Parse the result and check for empty / still-generating state.
        //
        // Placeholder: return an error describing what would happen.
        Err(AiReaderError::Browser(BrowserError::ConnectionError(
            format!(
                "Placeholder: would query the last element matching '{}' on platform '{}' \
                 via CDP Runtime.evaluate to extract AI response text.",
                response_selector, name
            ),
        )))
    }

    /// Type a prompt into the AI service's input field, click send, and wait
    /// for a response to appear (up to `timeout_ms`).
    ///
    /// # Privacy Note
    ///
    /// The prompt text is typed into the user's own browser session. It is
    /// **not** sent to any server controlled by this application.
    pub async fn send_and_read(
        &self,
        prompt: &str,
        timeout_ms: u64,
    ) -> Result<AiResponse, AiReaderError> {
        if !self.session.connected {
            return Err(AiReaderError::Browser(BrowserError::NotConnected));
        }

        let (name, input_selector, send_selector, response_selector) = match &self.platform {
            AiPlatform::Custom {
                name,
                response_selector,
                input_selector,
                send_selector,
            } => (
                name.clone(),
                input_selector.clone(),
                send_selector.clone(),
                response_selector.clone(),
            ),
        };

        // In a real implementation we would:
        // 1. session.type_text(tab_id, &input_selector, prompt)
        // 2. session.click(tab_id, &send_selector)
        // 3. session.wait_for_selector(tab_id, &response_selector, timeout_ms)
        // 4. session.get_element_text(tab_id, &response_selector)
        //
        // Placeholder:
        let _ = (prompt, timeout_ms);
        Err(AiReaderError::Browser(BrowserError::ConnectionError(
            format!(
                "Placeholder: would type prompt into '{}', click '{}', \
                 then wait for response in '{}' on platform '{}'.",
                input_selector, send_selector, response_selector, name
            ),
        )))
    }

    /// Check whether the AI platform page is loaded and ready to accept
    /// prompts (i.e. the input field is present in the DOM).
    pub async fn is_ready(&self) -> bool {
        if !self.session.connected {
            return false;
        }
        // Placeholder: would check for presence of input_selector in DOM.
        false
    }
}

/// Errors specific to reading AI responses.
#[derive(Debug, thiserror::Error)]
pub enum AiReaderError {
    #[error("Browser error: {0}")]
    Browser(#[from] BrowserError),

    #[error("No AI response found on page")]
    NoResponse,

    #[error("AI is still generating response")]
    StillGenerating,

    #[error("Platform not recognized: {0}")]
    UnknownPlatform(String),
}
