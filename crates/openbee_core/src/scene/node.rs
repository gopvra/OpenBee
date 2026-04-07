//! Scene node trait and base implementation.

use glam::Vec2;

use crate::render::renderer::{Camera, Renderer};

/// Classification of scene node types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneNodeType {
    /// Generic group node.
    Group,
    /// Renders an actor (sprite/animation).
    Actor,
    /// Renders HUD elements in screen space.
    Hud,
    /// Renders a tile map plane.
    TilePlane,
    /// Renders a background image or parallax layer.
    Background,
}

/// Trait for all scene graph nodes.
pub trait SceneNode {
    /// Update the node by delta time in seconds.
    fn update(&mut self, dt: f32);

    /// Render this node using the given renderer and camera.
    fn render(&self, renderer: &mut dyn Renderer, camera: &Camera);

    /// Z-ordering value; lower values are drawn first (further back).
    fn z_order(&self) -> i32;

    /// Whether this node is visible.
    fn is_visible(&self) -> bool;

    /// Get the world position of this node.
    fn position(&self) -> Vec2;

    /// Get the node type.
    fn node_type(&self) -> SceneNodeType;
}

/// A basic scene node with common fields. Can serve as a group node for children.
pub struct BaseSceneNode {
    /// Position in world-space pixels.
    pub pos: Vec2,
    /// Z-ordering depth.
    pub z: i32,
    /// Whether the node and its children should be rendered.
    pub visible: bool,
    /// Node type.
    pub ntype: SceneNodeType,
    /// Child nodes.
    pub children: Vec<Box<dyn SceneNode + Send>>,
}

impl BaseSceneNode {
    /// Create a new group node at the given position.
    pub fn new(position: Vec2, z_order: i32) -> Self {
        Self {
            pos: position,
            z: z_order,
            visible: true,
            ntype: SceneNodeType::Group,
            children: Vec::new(),
        }
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: Box<dyn SceneNode + Send>) {
        self.children.push(child);
    }

    /// Sort children by z-order for correct draw ordering.
    pub fn sort_children(&mut self) {
        self.children.sort_by_key(|c| c.z_order());
    }
}

impl SceneNode for BaseSceneNode {
    fn update(&mut self, dt: f32) {
        for child in &mut self.children {
            child.update(dt);
        }
    }

    fn render(&self, renderer: &mut dyn Renderer, camera: &Camera) {
        if !self.visible {
            return;
        }
        for child in &self.children {
            if child.is_visible() {
                child.render(renderer, camera);
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
        self.pos
    }

    fn node_type(&self) -> SceneNodeType {
        self.ntype
    }
}
