//! Advanced movement system: wall jump, double jump, dash, wall slide,
//! crouch/crawl, glide, ladder climbing, and swimming.

use glam::Vec2;
use openbee_core::ecs::{System, World};

use crate::components::health::HealthComponent;
use crate::components::kinematic::KinematicComponent;
use crate::components::movement::advanced::{
    CrouchComponent, DashComponent, DoubleJumpComponent, GlideComponent, LadderClimbComponent,
    SwimmingComponent, WallJumpComponent, WallSlideComponent,
};
use crate::components::physics::PhysicsComponent;

/// Processes all advanced movement abilities each frame.
pub struct AdvancedMovementSystem;

impl System for AdvancedMovementSystem {
    fn name(&self) -> &str {
        "AdvancedMovementSystem"
    }

    fn update(&mut self, world: &mut World, dt: f32) {
        let entities: Vec<_> = world.entities();

        for &entity in &entities {
            Self::update_wall_jump(world, entity, dt);
            Self::update_double_jump(world, entity, dt);
            Self::update_dash(world, entity, dt);
            Self::update_wall_slide(world, entity, dt);
            Self::update_crouch(world, entity, dt);
            Self::update_glide(world, entity, dt);
            Self::update_ladder_climb(world, entity, dt);
            Self::update_swimming(world, entity, dt);
        }
    }
}

impl AdvancedMovementSystem {
    // ── Wall Jump ──────────────────────────────────────────────────────

    fn update_wall_jump(world: &mut World, entity: openbee_core::ecs::Entity, dt: f32) {
        let on_ground = world
            .get_component::<KinematicComponent>(entity)
            .is_some_and(|k| k.on_ground);

        let wall_jump = match world.get_component_mut::<WallJumpComponent>(entity) {
            Some(wj) if wj.enabled => wj,
            _ => return,
        };

        // Tick cooldown.
        if wall_jump.cooldown_timer > 0.0 {
            wall_jump.cooldown_timer = (wall_jump.cooldown_timer - dt).max(0.0);
        }

        // Track wall-contact time.
        if wall_jump.wall_direction != 0 {
            wall_jump.wall_contact_time += dt;
        } else {
            wall_jump.wall_contact_time = 0.0;
        }

        // Reset wall-sliding state when on the ground.
        if on_ground {
            wall_jump.is_wall_sliding = false;
        }
    }

    // ── Double Jump ────────────────────────────────────────────────────

    fn update_double_jump(world: &mut World, entity: openbee_core::ecs::Entity, dt: f32) {
        let on_ground = world
            .get_component::<KinematicComponent>(entity)
            .is_some_and(|k| k.on_ground);

        let _ = dt; // not used directly; reset is frame-based.

        let dj = match world.get_component_mut::<DoubleJumpComponent>(entity) {
            Some(d) => d,
            None => return,
        };

        // Reset extra jumps when on the ground.
        if on_ground && dj.can_reset_on_ground {
            dj.jumps_remaining = dj.max_extra_jumps;
        }
    }

    // ── Dash ───────────────────────────────────────────────────────────

    fn update_dash(world: &mut World, entity: openbee_core::ecs::Entity, dt: f32) {
        // Read dash state.
        let (is_dashing, dash_timer, dash_duration, dash_dir, dash_speed, cooldown, invuln) = {
            let dash = match world.get_component_mut::<DashComponent>(entity) {
                Some(d) => d,
                None => return,
            };

            // Tick cooldown.
            if dash.cooldown_timer > 0.0 {
                dash.cooldown_timer = (dash.cooldown_timer - dt).max(0.0);
            }

            if dash.is_dashing {
                dash.dash_timer += dt;
                if dash.dash_timer >= dash.dash_duration {
                    dash.is_dashing = false;
                    dash.cooldown_timer = dash.dash_cooldown;
                }
            }

            (
                dash.is_dashing,
                dash.dash_timer,
                dash.dash_duration,
                dash.dash_direction,
                dash.dash_speed,
                dash.cooldown_timer,
                dash.invulnerable_during_dash,
            )
        };

        let _ = (dash_timer, dash_duration, cooldown);

        // Apply dash velocity.
        if is_dashing {
            if let Some(kin) = world.get_component_mut::<KinematicComponent>(entity) {
                let dir = Vec2::new(dash_dir.0, dash_dir.1);
                let dir = if dir.length_squared() > 0.0 {
                    dir.normalize()
                } else {
                    Vec2::X
                };
                kin.velocity = dir * dash_speed;
            }

            // Optionally make invulnerable while dashing.
            if invuln {
                if let Some(health) = world.get_component_mut::<HealthComponent>(entity) {
                    health.invulnerable = true;
                }
            }
        }
    }

    // ── Wall Slide ─────────────────────────────────────────────────────

    fn update_wall_slide(world: &mut World, entity: openbee_core::ecs::Entity, dt: f32) {
        let _ = dt;

        let fall_speed = world
            .get_component::<KinematicComponent>(entity)
            .map_or(0.0, |k| k.velocity.y);

        let ws = match world.get_component_mut::<WallSlideComponent>(entity) {
            Some(w) if w.can_wall_slide => w,
            _ => return,
        };

        // Only slide when falling fast enough.
        ws.is_sliding = fall_speed > ws.min_fall_speed_to_slide;

        // Cap downward velocity to slide speed.
        if ws.is_sliding {
            let slide_speed = world
                .get_component::<WallSlideComponent>(entity)
                .map_or(40.0, |w| w.slide_speed);
            if let Some(kin) = world.get_component_mut::<KinematicComponent>(entity) {
                if kin.velocity.y > 0.0 {
                    kin.velocity.y = kin.velocity.y.min(slide_speed);
                }
            }
        }
    }

    // ── Crouch / Crawl ─────────────────────────────────────────────────

    fn update_crouch(world: &mut World, entity: openbee_core::ecs::Entity, dt: f32) {
        let _ = dt;

        let is_crawling = {
            let crouch = match world.get_component::<CrouchComponent>(entity) {
                Some(c) => c,
                None => return,
            };
            crouch.is_crouching && crouch.can_crawl && crouch.is_crawling
        };

        // Limit speed while crawling.
        if is_crawling {
            let crawl_speed = world
                .get_component::<CrouchComponent>(entity)
                .map_or(60.0, |c| c.crawl_speed);
            if let Some(kin) = world.get_component_mut::<KinematicComponent>(entity) {
                kin.velocity.x = kin.velocity.x.clamp(-crawl_speed, crawl_speed);
            }
        }
    }

    // ── Glide ──────────────────────────────────────────────────────────

    fn update_glide(world: &mut World, entity: openbee_core::ecs::Entity, dt: f32) {
        let on_ground = world
            .get_component::<KinematicComponent>(entity)
            .is_some_and(|k| k.on_ground);

        let (is_gliding, gravity_scale) = {
            let glide = match world.get_component_mut::<GlideComponent>(entity) {
                Some(g) => g,
                None => return,
            };

            // Reset timer on ground.
            if on_ground {
                glide.glide_timer = 0.0;
                glide.is_gliding = false;
                return;
            }

            // Advance timer while gliding.
            if glide.is_gliding {
                glide.glide_timer += dt;
                if glide.glide_timer >= glide.max_glide_time {
                    glide.is_gliding = false;
                }
            }

            (glide.is_gliding, glide.glide_gravity_scale)
        };

        // Reduce gravity while gliding.
        if is_gliding {
            if let Some(phys) = world.get_component_mut::<PhysicsComponent>(entity) {
                phys.gravity_scale = gravity_scale;
            }
        }
    }

    // ── Ladder Climbing ────────────────────────────────────────────────

    fn update_ladder_climb(world: &mut World, entity: openbee_core::ecs::Entity, dt: f32) {
        let _ = dt;

        let (is_on_ladder, snap_distance) = {
            let ladder = match world.get_component::<LadderClimbComponent>(entity) {
                Some(l) => l,
                None => return,
            };
            (ladder.is_on_ladder, ladder.ladder_snap_distance)
        };

        if is_on_ladder {
            // Disable gravity while on a ladder.
            if let Some(phys) = world.get_component_mut::<PhysicsComponent>(entity) {
                phys.gravity_scale = 0.0;
            }

            // Zero out horizontal velocity so the entity stays on the ladder.
            if let Some(kin) = world.get_component_mut::<KinematicComponent>(entity) {
                kin.velocity.x = 0.0;
            }

            let _ = snap_distance; // would be used in full impl for X-axis snapping
        }
    }

    // ── Swimming ───────────────────────────────────────────────────────

    fn update_swimming(world: &mut World, entity: openbee_core::ecs::Entity, dt: f32) {
        let (is_submerged, buoyancy_force, breath_remaining, drown_damage_rate) = {
            let swim = match world.get_component_mut::<SwimmingComponent>(entity) {
                Some(s) => s,
                None => return,
            };

            if swim.is_submerged {
                // Deplete breath.
                swim.breath_remaining = (swim.breath_remaining - dt).max(0.0);
            } else if swim.breath_remaining < swim.breath_max {
                // Recover breath when not submerged (fast recovery).
                swim.breath_remaining =
                    (swim.breath_remaining + dt * 3.0).min(swim.breath_max);
            }

            (
                swim.is_submerged,
                swim.buoyancy_force,
                swim.breath_remaining,
                swim.drown_damage_rate,
            )
        };

        if is_submerged {
            // Apply buoyancy as upward acceleration.
            if let Some(kin) = world.get_component_mut::<KinematicComponent>(entity) {
                kin.acceleration.y -= buoyancy_force;
            }

            // Drowning damage when out of breath.
            if breath_remaining <= 0.0 {
                if let Some(health) = world.get_component_mut::<HealthComponent>(entity) {
                    let dmg = (drown_damage_rate * dt).ceil() as i32;
                    health.apply_damage(dmg);
                }
            }
        }
    }
}
