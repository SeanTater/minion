use crate::components::Player;
use crate::game_logic::movement::MovementCalculation;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

/// Debug logging utility for movement systems
pub struct MovementDebugger;

impl MovementDebugger {
    /// Log input event details
    pub fn log_input_event(
        cursor_pos: Vec2,
        camera_pos: Vec3,
        ray_origin: Vec3,
        ray_direction: Vec3,
    ) {
        info!(
            "INPUT: cursor({:.2}, {:.2}) camera({:.2}, {:.2}, {:.2}) ray_origin({:.2}, {:.2}, {:.2}) ray_dir({:.3}, {:.3}, {:.3})",
            cursor_pos.x,
            cursor_pos.y,
            camera_pos.x,
            camera_pos.y,
            camera_pos.z,
            ray_origin.x,
            ray_origin.y,
            ray_origin.z,
            ray_direction.x,
            ray_direction.y,
            ray_direction.z
        );
    }

    /// Log target calculation details
    pub fn log_target_calculation(
        player_pos: Vec3,
        player_y: f32,
        hit_point: Vec3,
        final_target: Vec3,
    ) {
        info!(
            "TARGET: player_pos({:.2}, {:.2}, {:.2}) player_y({:.2}) hit_point({:.2}, {:.2}, {:.2}) final_target({:.2}, {:.2}, {:.2})",
            player_pos.x,
            player_pos.y,
            player_pos.z,
            player_y,
            hit_point.x,
            hit_point.y,
            hit_point.z,
            final_target.x,
            final_target.y,
            final_target.z
        );
    }

    /// Log movement calculation details
    pub fn log_movement_calculation(
        player_pos: Vec3,
        target: Vec3,
        calculation: MovementCalculation,
    ) {
        info!(
            "MOVEMENT: pos({:.2}, {:.2}, {:.2}) target({:.2}, {:.2}, {:.2}) should_move({}) distance({:.2}) slowdown({:.2}) movement({:.3}, {:.3}, {:.3})",
            player_pos.x,
            player_pos.y,
            player_pos.z,
            target.x,
            target.y,
            target.z,
            calculation.should_move,
            calculation.distance_to_target,
            calculation.slowdown_factor,
            calculation.movement_vector.x,
            calculation.movement_vector.y,
            calculation.movement_vector.z
        );
    }

    /// Log character controller state
    pub fn log_controller_state(entity: Entity, controller: &KinematicCharacterController) {
        let translation = controller.translation.unwrap_or(Vec3::ZERO);
        info!(
            "CONTROLLER: entity({:?}) translation({:.3}, {:.3}, {:.3}) slide({}) snap_to_ground({:?}) offset({:?})",
            entity,
            translation.x,
            translation.y,
            translation.z,
            controller.slide,
            controller.snap_to_ground,
            controller.offset
        );
    }

    /// Log character controller output
    pub fn log_controller_output(entity: Entity, output: &KinematicCharacterControllerOutput) {
        info!(
            "OUTPUT: entity({:?}) effective_translation({:.3}, {:.3}, {:.3}) grounded({}) desired_translation({:.3}, {:.3}, {:.3})",
            entity,
            output.effective_translation.x,
            output.effective_translation.y,
            output.effective_translation.z,
            output.grounded,
            output.desired_translation.x,
            output.desired_translation.y,
            output.desired_translation.z
        );
    }

    /// Log transform changes
    pub fn log_transform_change(
        entity: Entity,
        old_transform: Transform,
        new_transform: Transform,
    ) {
        let pos_delta = new_transform.translation - old_transform.translation;
        info!(
            "TRANSFORM: entity({:?}) old_pos({:.3}, {:.3}, {:.3}) new_pos({:.3}, {:.3}, {:.3}) delta({:.6}, {:.6}, {:.6})",
            entity,
            old_transform.translation.x,
            old_transform.translation.y,
            old_transform.translation.z,
            new_transform.translation.x,
            new_transform.translation.y,
            new_transform.translation.z,
            pos_delta.x,
            pos_delta.y,
            pos_delta.z
        );
    }

    /// Log rotation changes
    pub fn log_rotation_change(
        entity: Entity,
        old_rotation: Quat,
        new_rotation: Quat,
        target: Option<Vec3>,
    ) {
        let angle_diff = old_rotation.angle_between(new_rotation);
        info!(
            "ROTATION: entity({:?}) angle_change({:.3} deg) target({:?})",
            entity,
            angle_diff.to_degrees(),
            target.map(|t| format!("({:.2}, {:.2}, {:.2})", t.x, t.y, t.z))
        );
    }

    /// Log physics state
    pub fn log_physics_state(entity: Entity, rigid_body: &RigidBody, velocity: Option<&Velocity>) {
        let vel_str = velocity
            .map(|v| {
                format!(
                    "linear({:.2}, {:.2}, {:.2}) angular({:.2}, {:.2}, {:.2})",
                    v.linvel.x, v.linvel.y, v.linvel.z, v.angvel.x, v.angvel.y, v.angvel.z
                )
            })
            .unwrap_or_else(|| "None".to_string());

        info!(
            "PHYSICS: entity({:?}) rigid_body({:?}) velocity({})",
            entity, rigid_body, vel_str
        );
    }

    /// Log system execution timing
    pub fn log_system_execution(system_name: &str, start_time: std::time::Instant) {
        let elapsed = start_time.elapsed();
        debug!(
            "TIMING: {} took {:.2}ms",
            system_name,
            elapsed.as_secs_f64() * 1000.0
        );
    }

    /// Log error conditions
    pub fn log_error(context: &str, error_msg: &str) {
        error!("ERROR in {}: {}", context, error_msg);
    }

    /// Log warning conditions
    pub fn log_warning(context: &str, warning_msg: &str) {
        warn!("WARNING in {}: {}", context, warning_msg);
    }

    /// Log frame-by-frame player state summary
    pub fn log_player_state_summary(
        entity: Entity,
        player: &Player,
        transform: &Transform,
        controller: &KinematicCharacterController,
        output: Option<&KinematicCharacterControllerOutput>,
    ) {
        let target_str = player
            .move_target
            .map(|t| format!("({:.2}, {:.2}, {:.2})", t.x, t.y, t.z))
            .unwrap_or_else(|| "None".to_string());

        let controller_translation = controller.translation.unwrap_or(Vec3::ZERO);

        let output_str = output
            .map(|o| {
                format!(
                    "effective({:.3}, {:.3}, {:.3}) grounded({})",
                    o.effective_translation.x,
                    o.effective_translation.y,
                    o.effective_translation.z,
                    o.grounded
                )
            })
            .unwrap_or_else(|| "None".to_string());

        info!(
            "PLAYER_STATE: entity({:?}) pos({:.3}, {:.3}, {:.3}) target({}) controller_move({:.3}, {:.3}, {:.3}) output({})",
            entity,
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
            target_str,
            controller_translation.x,
            controller_translation.y,
            controller_translation.z,
            output_str
        );
    }
}

/// Resource to control debug logging levels
#[derive(Resource)]
pub struct MovementDebugConfig {
    pub log_input: bool,
    pub log_targets: bool,
    pub log_calculations: bool,
    pub log_controller: bool,
    pub log_output: bool,
    pub log_transforms: bool,
    pub log_physics: bool,
    pub log_timing: bool,
    pub log_frame_summary: bool,
}

impl Default for MovementDebugConfig {
    fn default() -> Self {
        Self {
            log_input: true,
            log_targets: true,
            log_calculations: true,
            log_controller: true,
            log_output: true,
            log_transforms: true,
            log_physics: false,       // Can be noisy
            log_timing: false,        // Can be noisy
            log_frame_summary: false, // Can be very noisy
        }
    }
}

/// Macro for conditional debug logging
#[macro_export]
macro_rules! debug_log {
    ($config:expr, $field:ident, $($args:tt)*) => {
        if $config.$field {
            $($args)*
        }
    };
}
