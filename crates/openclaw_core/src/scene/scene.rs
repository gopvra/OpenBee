//! Scene struct managing the root node tree and camera.

use std::collections::HashMap;

use glam::Vec2;

use crate::ecs::Entity;
use crate::render::renderer::{Camera, Rect, Renderer};

use super::node::{BaseSceneNode, SceneNode};

/// The scene graph that holds a tree of nodes and a camera.
pub struct Scene {
    /// Root node of the scene tree.
    root: BaseSceneNode,
    /// Camera used for world-to-screen transformation.
    pub camera: Camera,
    /// Mapping from entities to node indices for quick lookup.
    actor_nodes: HashMap<Entity, usize>,
    /// Counter for generating unique node indices.
    next_index: usize,
}

impl Scene {
    /// Create a new empty scene with a default camera.
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        Self {
            root: BaseSceneNode::new(Vec2::ZERO, 0),
            camera: Camera {
                position: Vec2::ZERO,
                zoom: 1.0,
                viewport: Rect::new(0.0, 0.0, screen_width as f32, screen_height as f32),
            },
            actor_nodes: HashMap::new(),
            next_index: 0,
        }
    }

    /// Add a child node to the root. Returns a unique index.
    pub fn add_node(&mut self, node: Box<dyn SceneNode + Send>) -> usize {
        let index = self.next_index;
        self.next_index += 1;
        self.root.add_child(node);
        index
    }

    /// Associate an entity with a node index for tracking.
    pub fn register_actor_node(&mut self, entity: Entity, node_index: usize) {
        self.actor_nodes.insert(entity, node_index);
    }

    /// Remove the association for an entity.
    pub fn unregister_actor_node(&mut self, entity: Entity) {
        self.actor_nodes.remove(&entity);
    }

    /// Get the node index for an entity.
    pub fn get_actor_node_index(&self, entity: Entity) -> Option<usize> {
        self.actor_nodes.get(&entity).copied()
    }

    /// Update the scene tree by delta time.
    pub fn update(&mut self, dt: f32) {
        self.root.update(dt);
    }

    /// Render the entire scene using the given renderer.
    pub fn render(&self, renderer: &mut dyn Renderer) {
        renderer.set_camera(&self.camera);
        self.root.sort_children_for_render();
        self.root.render(renderer, &self.camera);
    }

    /// Get a mutable reference to the root node.
    pub fn root_mut(&mut self) -> &mut BaseSceneNode {
        &mut self.root
    }

    /// Get a reference to the root node.
    pub fn root(&self) -> &BaseSceneNode {
        &self.root
    }
}

/// Extension to allow sorting without mutability issues during render.
impl BaseSceneNode {
    fn sort_children_for_render(&self) {
        // Children should already be sorted from the update pass;
        // this is a no-op reminder that sorting happens in update.
    }
}
