//! Browser skill — web page reading and AI interaction.

use crate::skill::{Skill, SkillCommand};
use crate::task::{Task, TaskResult};

/// Handles browser automation: read web pages, interact with AI services.
pub struct BrowserSkill;

impl BrowserSkill {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BrowserSkill {
    fn default() -> Self {
        Self::new()
    }
}

impl Skill for BrowserSkill {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "Browser automation: read web pages, extract AI responses, interact with web AI"
    }

    fn keywords(&self) -> &[&str] {
        &[
            "browser",
            "web",
            "page",
            "read",
            "scrape",
            "ai",
            "chat",
            "url",
            "open",
            "navigate",
            "search",
            "浏览器",
            "网页",
            "读取",
            "打开",
            "搜索",
        ]
    }

    fn execute(&self, task: &Task) -> TaskResult {
        let lower = task.instruction.to_lowercase();

        if lower.contains("read") || lower.contains("读取") {
            TaskResult::success(
                "Browser read requires Chrome running with --remote-debugging-port=9222. \
                 The browser AI reader can extract content from any page you're logged into.",
            )
            .with_suggestions(vec![
                "Start Chrome with: chrome --remote-debugging-port=9222".to_string(),
                "Then use the BrowserConnector API to read page content".to_string(),
            ])
        } else if lower.contains("navigate") || lower.contains("open") || lower.contains("打开") {
            TaskResult::success(
                "Navigation requires an active browser session via CDP (Chrome DevTools Protocol).",
            )
        } else {
            TaskResult::success(
                "Browser skill: can read web pages, extract AI responses, navigate. \
                 Requires Chrome with --remote-debugging-port=9222.",
            )
        }
    }

    fn commands(&self) -> Vec<SkillCommand> {
        vec![
            SkillCommand {
                name: "read".to_string(),
                description: "Read content from a web page".to_string(),
                example: "read the AI response from the current page".to_string(),
            },
            SkillCommand {
                name: "navigate".to_string(),
                description: "Open a URL in the browser".to_string(),
                example: "open https://example.com".to_string(),
            },
        ]
    }
}
