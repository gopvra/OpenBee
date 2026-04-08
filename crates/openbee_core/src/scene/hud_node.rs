//! HUD scene node for rendering screen-space UI elements.

use glam::Vec2;

use crate::render::renderer::{Camera, Color, Rect, Renderer};

use super::node::{SceneNode, SceneNodeType};

/// A single HUD element to display.
#[derive(Debug, Clone)]
pub enum HudElement {
    /// Health bar: current and max values.
    HealthBar {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        current: f32,
        max: f32,
    },
    /// Score display.
    Score { x: f32, y: f32, value: u32 },
    /// Lives remaining display.
    Lives { x: f32, y: f32, count: i32 },
    /// Ammo count for the current weapon.
    Ammo {
        x: f32,
        y: f32,
        pistol: i32,
        magic: i32,
        dynamite: i32,
    },
    /// Custom text label.
    Text {
        x: f32,
        y: f32,
        text: String,
        size: f32,
        color: Color,
    },
}

/// Scene node that renders HUD elements in screen-space (ignoring the camera).
pub struct HudSceneNode {
    /// Z-order (usually high to draw on top).
    pub z: i32,
    /// Whether the HUD is visible.
    pub visible: bool,
    /// HUD elements to render.
    pub elements: Vec<HudElement>,
}

impl HudSceneNode {
    /// Create a new HUD node.
    pub fn new(z_order: i32) -> Self {
        Self {
            z: z_order,
            visible: true,
            elements: Vec::new(),
        }
    }

    /// Add a HUD element.
    pub fn add_element(&mut self, element: HudElement) {
        self.elements.push(element);
    }

    /// Clear all HUD elements.
    pub fn clear(&mut self) {
        self.elements.clear();
    }

    /// Update a score element if present.
    pub fn set_score(&mut self, value: u32) {
        for elem in &mut self.elements {
            if let HudElement::Score { value: v, .. } = elem {
                *v = value;
            }
        }
    }

    /// Update a lives element if present.
    pub fn set_lives(&mut self, count: i32) {
        for elem in &mut self.elements {
            if let HudElement::Lives { count: c, .. } = elem {
                *c = count;
            }
        }
    }

    /// Update the health bar if present.
    pub fn set_health(&mut self, current: f32, max: f32) {
        for elem in &mut self.elements {
            if let HudElement::HealthBar {
                current: c, max: m, ..
            } = elem
            {
                *c = current;
                *m = max;
            }
        }
    }
}

impl SceneNode for HudSceneNode {
    fn update(&mut self, _dt: f32) {
        // HUD elements are updated externally via set_score, set_lives, etc.
    }

    fn render(&self, renderer: &mut dyn Renderer, _camera: &Camera) {
        if !self.visible {
            return;
        }

        for element in &self.elements {
            match element {
                HudElement::HealthBar {
                    x,
                    y,
                    width,
                    height,
                    current,
                    max,
                } => {
                    // Background (dark red).
                    renderer
                        .draw_rect_filled(Rect::new(*x, *y, *width, *height), Color::rgb(80, 0, 0));
                    // Foreground (green to red gradient based on health).
                    let ratio = if *max > 0.0 {
                        (*current / *max).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let bar_color =
                        Color::rgb((255.0 * (1.0 - ratio)) as u8, (255.0 * ratio) as u8, 0);
                    renderer
                        .draw_rect_filled(Rect::new(*x, *y, *width * ratio, *height), bar_color);
                    // Border.
                    renderer.draw_rect_outline(Rect::new(*x, *y, *width, *height), Color::WHITE);
                }
                HudElement::Score { x, y, value } => {
                    renderer.draw_text(
                        &format!("SCORE: {}", value),
                        Vec2::new(*x, *y),
                        16.0,
                        Color::WHITE,
                    );
                }
                HudElement::Lives { x, y, count } => {
                    renderer.draw_text(
                        &format!("LIVES: {}", count),
                        Vec2::new(*x, *y),
                        16.0,
                        Color::WHITE,
                    );
                }
                HudElement::Ammo {
                    x,
                    y,
                    pistol,
                    magic,
                    dynamite,
                } => {
                    renderer.draw_text(
                        &format!("P:{} M:{} D:{}", pistol, magic, dynamite),
                        Vec2::new(*x, *y),
                        14.0,
                        Color::WHITE,
                    );
                }
                HudElement::Text {
                    x,
                    y,
                    text,
                    size,
                    color,
                } => {
                    renderer.draw_text(text, Vec2::new(*x, *y), *size, *color);
                }
            }
        }
    }

    fn z_order(&self) -> i32 {
        self.z
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn position(&self) -> Vec2 {
        Vec2::ZERO
    }

    fn node_type(&self) -> SceneNodeType {
        SceneNodeType::Hud
    }
}
