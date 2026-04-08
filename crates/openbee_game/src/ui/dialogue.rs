//! Dialogue system for NPC conversations and story delivery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single node in a dialogue tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: String,
    pub speaker: String,
    pub text: String,
    pub portrait: Option<String>,
    pub choices: Vec<DialogueChoice>,
    pub next: Option<String>,
    pub condition: Option<String>,
    pub on_enter_action: Option<String>,
}

/// A choice the player can make during dialogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub next_node: String,
    pub condition: Option<String>,
    pub set_flag: Option<(String, bool)>,
}

/// A complete dialogue tree with multiple nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTree {
    pub id: String,
    pub nodes: HashMap<String, DialogueNode>,
    pub start_node: String,
}

/// Manages dialogue playback, typewriter text, and choices.
pub struct DialogueManager {
    pub trees: HashMap<String, DialogueTree>,
    pub current_tree: Option<String>,
    pub current_node: Option<String>,
    pub is_active: bool,
    pub display_text: String,
    pub display_speaker: String,
    pub display_portrait: Option<String>,
    pub display_choices: Vec<String>,
    pub text_speed: f32,
    pub chars_displayed: f32,
    pub flags: HashMap<String, bool>,
}

impl Default for DialogueManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueManager {
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            current_tree: None,
            current_node: None,
            is_active: false,
            display_text: String::new(),
            display_speaker: String::new(),
            display_portrait: None,
            display_choices: Vec::new(),
            text_speed: 30.0,
            chars_displayed: 0.0,
            flags: HashMap::new(),
        }
    }

    pub fn load_tree(&mut self, tree: DialogueTree) {
        self.trees.insert(tree.id.clone(), tree);
    }

    pub fn start_dialogue(&mut self, tree_id: &str) -> bool {
        if let Some(tree) = self.trees.get(tree_id) {
            let start = tree.start_node.clone();
            self.current_tree = Some(tree_id.to_string());
            self.current_node = Some(start.clone());
            self.is_active = true;
            self.chars_displayed = 0.0;
            self.load_node(&start);
            true
        } else {
            false
        }
    }

    fn load_node(&mut self, node_id: &str) {
        let tree_id = match &self.current_tree {
            Some(id) => id.clone(),
            None => return,
        };
        if let Some(tree) = self.trees.get(&tree_id) {
            if let Some(node) = tree.nodes.get(node_id) {
                self.display_speaker = node.speaker.clone();
                self.display_text = node.text.clone();
                self.display_portrait = node.portrait.clone();
                self.display_choices = node
                    .choices
                    .iter()
                    .filter(|c| {
                        c.condition
                            .as_ref()
                            .map(|flag| self.flags.get(flag).copied().unwrap_or(false))
                            .unwrap_or(true)
                    })
                    .map(|c| c.text.clone())
                    .collect();
                self.chars_displayed = 0.0;
            }
        }
    }

    pub fn advance(&mut self) -> bool {
        if !self.is_active {
            return false;
        }
        if !self.is_text_complete() {
            self.skip_text();
            return true;
        }

        let tree_id = match &self.current_tree {
            Some(id) => id.clone(),
            None => {
                self.is_active = false;
                return false;
            }
        };
        let node_id = match &self.current_node {
            Some(id) => id.clone(),
            None => {
                self.is_active = false;
                return false;
            }
        };

        if let Some(tree) = self.trees.get(&tree_id) {
            if let Some(node) = tree.nodes.get(&node_id) {
                if !node.choices.is_empty() {
                    return true; // waiting for choice
                }
                if let Some(next) = &node.next {
                    let next = next.clone();
                    self.current_node = Some(next.clone());
                    self.load_node(&next);
                    return true;
                }
            }
        }

        self.is_active = false;
        self.current_tree = None;
        self.current_node = None;
        false
    }

    pub fn choose(&mut self, index: usize) {
        let tree_id = match &self.current_tree {
            Some(id) => id.clone(),
            None => return,
        };
        let node_id = match &self.current_node {
            Some(id) => id.clone(),
            None => return,
        };

        if let Some(tree) = self.trees.get(&tree_id) {
            if let Some(node) = tree.nodes.get(&node_id) {
                let visible_choices: Vec<_> = node
                    .choices
                    .iter()
                    .filter(|c| {
                        c.condition
                            .as_ref()
                            .map(|flag| self.flags.get(flag).copied().unwrap_or(false))
                            .unwrap_or(true)
                    })
                    .collect();

                if let Some(choice) = visible_choices.get(index) {
                    if let Some((flag, value)) = &choice.set_flag {
                        self.flags.insert(flag.clone(), *value);
                    }
                    let next = choice.next_node.clone();
                    self.current_node = Some(next.clone());
                    self.load_node(&next);
                }
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.is_active && !self.is_text_complete() {
            self.chars_displayed += self.text_speed * dt;
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_text_complete(&self) -> bool {
        self.chars_displayed >= self.display_text.len() as f32
    }

    pub fn skip_text(&mut self) {
        self.chars_displayed = self.display_text.len() as f32;
    }

    pub fn visible_text(&self) -> &str {
        let end = (self.chars_displayed as usize).min(self.display_text.len());
        &self.display_text[..end]
    }

    pub fn get_flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }

    pub fn set_flag(&mut self, name: &str, value: bool) {
        self.flags.insert(name.to_string(), value);
    }
}
