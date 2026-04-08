//! Actor factory: creates entities from XML template definitions.

use anyhow::{Context, Result};
use glam::Vec2;
use openbee_core::ecs::{Entity, World};
use quick_xml::events::Event;
use quick_xml::Reader;
use rustc_hash::FxHashMap;
use tracing::{debug, warn};

use crate::components::animation::AnimationComponent;
use crate::components::collision::CollisionComponent;
use crate::components::health::HealthComponent;
use crate::components::kinematic::KinematicComponent;
use crate::components::physics::{PhysicsBodyType, PhysicsComponent};
use crate::components::render::RenderComponent;
use crate::components::transform::TransformComponent;

/// A parsed actor template that can stamp out entities.
#[derive(Debug, Clone)]
pub struct ActorTemplate {
    pub name: String,
    pub logic_type: String,
    pub image_set: String,
    pub animation_set: String,
    pub health: Option<i32>,
    pub damage: Option<i32>,
    pub speed: Option<f32>,
    pub body_type: PhysicsBodyType,
    pub hit_rect: [f32; 4],
    pub attack_rect: [f32; 4],
    pub z_order: i32,
    pub properties: FxHashMap<String, String>,
}

impl Default for ActorTemplate {
    fn default() -> Self {
        Self {
            name: String::new(),
            logic_type: String::new(),
            image_set: String::new(),
            animation_set: String::new(),
            health: None,
            damage: None,
            speed: None,
            body_type: PhysicsBodyType::Dynamic,
            hit_rect: [0.0; 4],
            attack_rect: [0.0; 4],
            z_order: 0,
            properties: FxHashMap::default(),
        }
    }
}

/// Registry of named actor templates loaded from XML data.
pub struct ActorTemplateRegistry {
    templates: FxHashMap<String, ActorTemplate>,
}

impl ActorTemplateRegistry {
    /// Create an empty template registry.
    pub fn new() -> Self {
        Self {
            templates: FxHashMap::default(),
        }
    }

    /// Load templates from an XML string.
    pub fn load_from_xml(&mut self, xml_data: &str) -> Result<()> {
        let mut reader = Reader::from_str(xml_data);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut current_template: Option<ActorTemplate> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    match tag_name.as_str() {
                        "Actor" => {
                            let mut template = ActorTemplate::default();
                            for attr in e.attributes().flatten() {
                                let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                let value = String::from_utf8_lossy(&attr.value).to_string();
                                match key.as_str() {
                                    "name" => template.name = value,
                                    "logic" => template.logic_type = value,
                                    "imageSet" => template.image_set = value,
                                    "animationSet" => template.animation_set = value,
                                    "zOrder" => template.z_order = value.parse().unwrap_or(0),
                                    _ => {
                                        template.properties.insert(key, value);
                                    }
                                }
                            }
                            current_template = Some(template);
                        }
                        "Health" => {
                            if let Some(ref mut t) = current_template {
                                for attr in e.attributes().flatten() {
                                    let key =
                                        String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    if key == "value" {
                                        t.health = value.parse().ok();
                                    }
                                }
                            }
                        }
                        "Damage" => {
                            if let Some(ref mut t) = current_template {
                                for attr in e.attributes().flatten() {
                                    let key =
                                        String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    if key == "value" {
                                        t.damage = value.parse().ok();
                                    }
                                }
                            }
                        }
                        "Speed" => {
                            if let Some(ref mut t) = current_template {
                                for attr in e.attributes().flatten() {
                                    let key =
                                        String::from_utf8_lossy(attr.key.as_ref()).to_string();
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    if key == "value" {
                                        t.speed = value.parse().ok();
                                    }
                                }
                            }
                        }
                        "HitRect" => {
                            if let Some(ref mut t) = current_template {
                                let mut idx = 0;
                                for attr in e.attributes().flatten() {
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    if idx < 4 {
                                        t.hit_rect[idx] = value.parse().unwrap_or(0.0);
                                        idx += 1;
                                    }
                                }
                            }
                        }
                        "AttackRect" => {
                            if let Some(ref mut t) = current_template {
                                let mut idx = 0;
                                for attr in e.attributes().flatten() {
                                    let value = String::from_utf8_lossy(&attr.value).to_string();
                                    if idx < 4 {
                                        t.attack_rect[idx] = value.parse().unwrap_or(0.0);
                                        idx += 1;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(ref e)) => {
                    let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag_name == "Actor" {
                        if let Some(template) = current_template.take() {
                            if !template.name.is_empty() {
                                debug!("Loaded actor template: {}", template.name);
                                self.templates.insert(template.name.clone(), template);
                            } else {
                                warn!("Skipping unnamed actor template");
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(anyhow::anyhow!("XML parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }

        Ok(())
    }

    /// Look up a template by name.
    pub fn get(&self, name: &str) -> Option<&ActorTemplate> {
        self.templates.get(name)
    }

    /// Spawn an entity from a named template at the given position.
    pub fn spawn(&self, world: &mut World, template_name: &str, position: Vec2) -> Result<Entity> {
        let template = self
            .templates
            .get(template_name)
            .with_context(|| format!("Unknown actor template: {}", template_name))?;

        let entity = world.create_entity();

        // Transform
        world.add_component(
            entity,
            TransformComponent {
                position,
                rotation: 0.0,
                scale: Vec2::ONE,
            },
        );

        // Render
        world.add_component(
            entity,
            RenderComponent {
                sprite_id: Some(template.image_set.clone()),
                visible: true,
                flip_x: false,
                z_order: template.z_order,
                color_mod: [255, 255, 255, 255],
            },
        );

        // Animation
        world.add_component(
            entity,
            AnimationComponent {
                current_animation: "idle".to_string(),
                animations: Default::default(),
                playing: true,
                speed: 1.0,
            },
        );

        // Physics
        world.add_component(
            entity,
            PhysicsComponent {
                body_handle: None,
                body_type: template.body_type,
                gravity_scale: 1.0,
            },
        );

        // Kinematic
        world.add_component(
            entity,
            KinematicComponent {
                velocity: Vec2::ZERO,
                acceleration: Vec2::ZERO,
                max_speed: Vec2::new(template.speed.unwrap_or(200.0), 800.0),
                on_ground: false,
            },
        );

        // Collision
        let hr = template.hit_rect;
        let ar = template.attack_rect;
        world.add_component(
            entity,
            CollisionComponent {
                hit_rect: openbee_core::render::Rect::new(hr[0], hr[1], hr[2], hr[3]),
                attack_rect: openbee_core::render::Rect::new(ar[0], ar[1], ar[2], ar[3]),
                clip_rect: openbee_core::render::Rect::new(0.0, 0.0, 0.0, 0.0),
                collision_mask: 0xFFFFFFFF,
                collision_layer: 1,
            },
        );

        // Health (if specified)
        if let Some(hp) = template.health {
            world.add_component(
                entity,
                HealthComponent {
                    current: hp,
                    max: hp,
                    invulnerable: false,
                    invulnerability_timer: 0.0,
                },
            );
        }

        debug!(
            "Spawned actor '{}' as {:?} at ({}, {})",
            template_name, entity, position.x, position.y
        );
        Ok(entity)
    }

    /// Return the number of loaded templates.
    pub fn template_count(&self) -> usize {
        self.templates.len()
    }

    /// Return all template names.
    pub fn template_names(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ActorTemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}
