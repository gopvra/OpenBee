//! # OpenBee Browser
//!
//! Browser automation for reading AI web content via the user's existing
//! logged-in browser session.
//!
//! ## Architecture
//!
//! This crate connects to a running Chrome/Chromium/Edge instance through the
//! **Chrome DevTools Protocol (CDP)** on a local debug port. The user starts
//! their browser with `--remote-debugging-port=9222`, logs into any web-based
//! AI service, and this crate can then read (but not modify) the AI responses.
//!
//! ## Privacy & Safety
//!
//! - **Read-only by default** — this crate extracts text from pages the user
//!   is already viewing. It does NOT inject scripts that modify page content.
//! - **No credential access** — this crate NEVER reads, stores, or transmits
//!   cookies, passwords, tokens, or any authentication material.
//! - **Local only** — all communication is with `localhost`; nothing is sent
//!   to any remote server.
//! - **No `unsafe` code** — the entire crate is safe Rust.

pub mod ai_reader;
pub mod browser;
pub mod page_parser;
pub mod session;
