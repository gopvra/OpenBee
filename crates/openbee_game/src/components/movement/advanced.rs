//! Advanced movement components: wall jump, double jump, dash, wall slide,
//! crouch/crawl, glide, ladder climbing, and swimming.

use openbee_core::ecs::Component;
use serde::{Deserialize, Serialize};

/// Wall jump – lets the player jump off walls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallJumpComponent {
    pub enabled: bool,
    pub wall_slide_speed: f32,
    pub wall_jump_force_x: f32,
    pub wall_jump_force_y: f32,
    /// How long the entity has been touching a wall (seconds).
    pub wall_contact_time: f32,
    pub wall_jump_cooldown: f32,
    pub cooldown_timer: f32,
    pub is_wall_sliding: bool,
    /// -1 left, 0 none, 1 right.
    pub wall_direction: i8,
}

impl Default for WallJumpComponent {
    fn default() -> Self {
        Self {
            enabled: true,
            wall_slide_speed: 50.0,
            wall_jump_force_x: 300.0,
            wall_jump_force_y: -400.0,
            wall_contact_time: 0.0,
            wall_jump_cooldown: 0.25,
            cooldown_timer: 0.0,
            is_wall_sliding: false,
            wall_direction: 0,
        }
    }
}

impl Component for WallJumpComponent {}

/// Double jump (and multi-jump).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleJumpComponent {
    pub max_extra_jumps: u32,
    pub jumps_remaining: u32,
    pub extra_jump_force: f32,
    pub can_reset_on_ground: bool,
}

impl Default for DoubleJumpComponent {
    fn default() -> Self {
        Self {
            max_extra_jumps: 1,
            jumps_remaining: 1,
            extra_jump_force: -350.0,
            can_reset_on_ground: true,
        }
    }
}

impl Component for DoubleJumpComponent {}

/// Dash ability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashComponent {
    pub dash_speed: f32,
    pub dash_duration: f32,
    pub dash_cooldown: f32,
    pub dash_timer: f32,
    pub cooldown_timer: f32,
    pub is_dashing: bool,
    pub dash_direction: (f32, f32),
    pub can_dash_in_air: bool,
    pub invulnerable_during_dash: bool,
}

impl Default for DashComponent {
    fn default() -> Self {
        Self {
            dash_speed: 600.0,
            dash_duration: 0.2,
            dash_cooldown: 0.8,
            dash_timer: 0.0,
            cooldown_timer: 0.0,
            is_dashing: false,
            dash_direction: (1.0, 0.0),
            can_dash_in_air: true,
            invulnerable_during_dash: true,
        }
    }
}

impl Component for DashComponent {}

/// Wall slide – slows descent when touching a wall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallSlideComponent {
    pub slide_speed: f32,
    pub can_wall_slide: bool,
    pub is_sliding: bool,
    pub min_fall_speed_to_slide: f32,
}

impl Default for WallSlideComponent {
    fn default() -> Self {
        Self {
            slide_speed: 40.0,
            can_wall_slide: true,
            is_sliding: false,
            min_fall_speed_to_slide: 10.0,
        }
    }
}

impl Component for WallSlideComponent {}

/// Crouch and crawl.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrouchComponent {
    pub is_crouching: bool,
    /// Height multiplier while crouching, e.g. 0.5 = half height.
    pub crouch_height_multiplier: f32,
    pub crawl_speed: f32,
    pub can_crawl: bool,
    pub is_crawling: bool,
    /// Whether standing up is blocked by a ceiling above.
    pub stand_up_blocked: bool,
}

impl Default for CrouchComponent {
    fn default() -> Self {
        Self {
            is_crouching: false,
            crouch_height_multiplier: 0.5,
            crawl_speed: 60.0,
            can_crawl: true,
            is_crawling: false,
            stand_up_blocked: false,
        }
    }
}

impl Component for CrouchComponent {}

/// Glide – slow fall with horizontal air control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlideComponent {
    /// Gravity scale while gliding, e.g. 0.2 for slow fall.
    pub glide_gravity_scale: f32,
    pub max_glide_time: f32,
    pub glide_timer: f32,
    pub is_gliding: bool,
    /// Horizontal air-control multiplier while gliding.
    pub horizontal_control: f32,
}

impl Default for GlideComponent {
    fn default() -> Self {
        Self {
            glide_gravity_scale: 0.2,
            max_glide_time: 3.0,
            glide_timer: 0.0,
            is_gliding: false,
            horizontal_control: 0.8,
        }
    }
}

impl Component for GlideComponent {}

/// Ladder climbing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LadderClimbComponent {
    pub climb_speed: f32,
    pub is_on_ladder: bool,
    pub can_jump_off_ladder: bool,
    pub ladder_top_dismount: bool,
    pub ladder_snap_distance: f32,
}

impl Default for LadderClimbComponent {
    fn default() -> Self {
        Self {
            climb_speed: 120.0,
            is_on_ladder: false,
            can_jump_off_ladder: true,
            ladder_top_dismount: true,
            ladder_snap_distance: 8.0,
        }
    }
}

impl Component for LadderClimbComponent {}

/// Swimming in water zones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwimmingComponent {
    pub swim_speed: f32,
    pub surface_bob_speed: f32,
    pub is_submerged: bool,
    pub is_on_surface: bool,
    pub breath_max: f32,
    pub breath_remaining: f32,
    pub drown_damage_rate: f32,
    pub buoyancy_force: f32,
}

impl Default for SwimmingComponent {
    fn default() -> Self {
        Self {
            swim_speed: 100.0,
            surface_bob_speed: 30.0,
            is_submerged: false,
            is_on_surface: false,
            breath_max: 10.0,
            breath_remaining: 10.0,
            drown_damage_rate: 5.0,
            buoyancy_force: 200.0,
        }
    }
}

impl Component for SwimmingComponent {}
