//! Player input mapping to character behaviors.

use openclaw_core::ecs::{Entity, World};

use crate::components::ammo::AmmoComponent;
use crate::components::controllable::ControllableComponent;
use crate::components::kinematic::KinematicComponent;

/// Input actions the player can perform, mapped from keyboard/gamepad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerAction {
    MoveLeft,
    MoveRight,
    Jump,
    Duck,
    Attack,
    FirePistol,
    ThrowDynamite,
    CastMagic,
    LookUp,
    LookDown,
    Pause,
}

/// State of all player input for the current frame.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub move_left: bool,
    pub move_right: bool,
    pub jump: bool,
    pub duck: bool,
    pub attack: bool,
    pub fire_pistol: bool,
    pub throw_dynamite: bool,
    pub cast_magic: bool,
    pub look_up: bool,
    pub look_down: bool,
    pub pause: bool,
    /// Whether jump was just pressed this frame (edge-triggered).
    pub jump_pressed: bool,
    /// Whether attack was just pressed this frame (edge-triggered).
    pub attack_pressed: bool,
}

/// Player character constants.
pub struct PlayerConstants {
    pub move_speed: f32,
    pub jump_force: f32,
    pub duck_speed_multiplier: f32,
    pub max_fall_speed: f32,
    pub coyote_time: f32,
    pub jump_buffer_time: f32,
}

impl Default for PlayerConstants {
    fn default() -> Self {
        Self {
            move_speed: 250.0,
            jump_force: -500.0,
            duck_speed_multiplier: 0.5,
            max_fall_speed: 800.0,
            coyote_time: 0.08,
            jump_buffer_time: 0.1,
        }
    }
}

/// Controller that translates player input into character movement and actions.
pub struct ActorController {
    pub player_entity: Option<Entity>,
    pub input_state: InputState,
    pub constants: PlayerConstants,
    /// Time since the player left the ground (for coyote time).
    coyote_timer: f32,
    /// Time since jump was pressed (for jump buffering).
    jump_buffer_timer: f32,
    /// Whether a jump is currently in progress.
    is_jumping: bool,
}

impl ActorController {
    /// Create a new controller not yet attached to a player entity.
    pub fn new() -> Self {
        Self {
            player_entity: None,
            input_state: InputState::default(),
            constants: PlayerConstants::default(),
            coyote_timer: 0.0,
            jump_buffer_timer: 0.0,
            is_jumping: false,
        }
    }

    /// Attach the controller to a player entity.
    pub fn set_player(&mut self, entity: Entity) {
        self.player_entity = Some(entity);
    }

    /// Update the player character based on current input state.
    pub fn update(&mut self, world: &mut World, dt: f32) {
        let entity = match self.player_entity {
            Some(e) => e,
            None => return,
        };

        if !world.is_alive(entity) {
            return;
        }

        // Check if the player can be controlled
        let can_control = world
            .get_component::<ControllableComponent>(entity)
            .map_or(false, |c| c.is_active && c.can_move);

        if !can_control {
            return;
        }

        // Update coyote time and jump buffer
        let on_ground = world
            .get_component::<KinematicComponent>(entity)
            .map_or(false, |k| k.on_ground);

        if on_ground {
            self.coyote_timer = self.constants.coyote_time;
            self.is_jumping = false;
        } else {
            self.coyote_timer = (self.coyote_timer - dt).max(0.0);
        }

        if self.input_state.jump_pressed {
            self.jump_buffer_timer = self.constants.jump_buffer_time;
        } else {
            self.jump_buffer_timer = (self.jump_buffer_timer - dt).max(0.0);
        }

        // Horizontal movement
        let mut move_x = 0.0;
        if self.input_state.move_left {
            move_x -= self.constants.move_speed;
        }
        if self.input_state.move_right {
            move_x += self.constants.move_speed;
        }
        if self.input_state.duck {
            move_x *= self.constants.duck_speed_multiplier;
        }

        // Read can_jump before taking the mutable borrow on kinematic.
        let can_jump = world
            .get_component::<ControllableComponent>(entity)
            .map_or(false, |c| c.can_jump);

        if let Some(kinematic) = world.get_component_mut::<KinematicComponent>(entity) {
            kinematic.velocity.x = move_x;

            if can_jump && self.jump_buffer_timer > 0.0 && self.coyote_timer > 0.0 && !self.is_jumping {
                kinematic.velocity.y = self.constants.jump_force;
                self.is_jumping = true;
                self.coyote_timer = 0.0;
                self.jump_buffer_timer = 0.0;
            }

            // Clamp fall speed
            if kinematic.velocity.y > self.constants.max_fall_speed {
                kinematic.velocity.y = self.constants.max_fall_speed;
            }
        }

        // Handle attacks
        if self.input_state.attack_pressed {
            let can_attack = world
                .get_component::<ControllableComponent>(entity)
                .map_or(false, |c| c.can_attack);
            if can_attack {
                // Attack initiated - combat system handles the rest
            }
        }

        // Handle ranged attacks (pistol, dynamite, magic) based on ammo
        if self.input_state.fire_pistol {
            if let Some(ammo) = world.get_component_mut::<AmmoComponent>(entity) {
                if ammo.pistol > 0 {
                    ammo.pistol -= 1;
                    // Projectile spawning handled by projectile system
                }
            }
        }

        if self.input_state.throw_dynamite {
            if let Some(ammo) = world.get_component_mut::<AmmoComponent>(entity) {
                if ammo.dynamite > 0 {
                    ammo.dynamite -= 1;
                }
            }
        }

        if self.input_state.cast_magic {
            if let Some(ammo) = world.get_component_mut::<AmmoComponent>(entity) {
                if ammo.magic > 0 {
                    ammo.magic -= 1;
                }
            }
        }
    }

    /// Reset all input state (called at the start of each frame before polling).
    pub fn clear_input(&mut self) {
        self.input_state.jump_pressed = false;
        self.input_state.attack_pressed = false;
    }
}

impl Default for ActorController {
    fn default() -> Self {
        Self::new()
    }
}
