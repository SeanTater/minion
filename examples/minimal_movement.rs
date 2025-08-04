/// Minimal KinematicCharacterController movement example
/// This demonstrates the absolute bare minimum needed for character movement
/// Run with: cargo run --example minimal_movement
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
struct SimplePlayer {
    target: Option<Vec3>,
    speed: f32,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Minimal Movement Test".into(),
                    resolution: (800., 600.).into(),
                    ..default()
                }),
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, move_character, log_state))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, -0.5, 0.0)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        RigidBody::Fixed,
        Collider::cuboid(10.0, 0.1, 10.0),
    ));

    // Simple character - capsule with kinematic controller
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.5, 2.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_translation(Vec3::new(0.0, 1.5, 0.0)),
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(1.0, 0.5),
        KinematicCharacterController {
            snap_to_ground: Some(CharacterLength::Absolute(2.0)),
            offset: CharacterLength::Absolute(0.01),
            slide: true,
            ..default()
        },
        SimplePlayer {
            target: None,
            speed: 5.0,
        },
    ));

    println!("Setup complete. Click anywhere to move the character.");
    println!("Red capsule should move to clicked positions.");
}

fn handle_input(
    mut player_query: Query<(&Transform, &mut SimplePlayer)>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        let Ok(window) = windows.single() else { return };
        let Some(cursor_pos) = window.cursor_position() else {
            return;
        };
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };
        let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else {
            return;
        };

        // Simple ground plane intersection (Y = 0)
        if ray.direction.y != 0.0 {
            let t = -ray.origin.y / ray.direction.y;
            if t > 0.0 {
                let hit_point = ray.origin + ray.direction * t;

                for (transform, mut player) in player_query.iter_mut() {
                    player.target = Some(hit_point);
                    println!(
                        "Click: New target set to ({:.2}, {:.2}, {:.2})",
                        hit_point.x, hit_point.y, hit_point.z
                    );
                    println!(
                        "Player position: ({:.2}, {:.2}, {:.2})",
                        transform.translation.x, transform.translation.y, transform.translation.z
                    );
                }
            }
        }
    }
}

fn move_character(
    mut query: Query<(
        &mut Transform,
        &mut SimplePlayer,
        &mut KinematicCharacterController,
    )>,
    time: Res<Time>,
) {
    for (transform, mut player, mut controller) in query.iter_mut() {
        if let Some(target) = player.target {
            // Calculate 2D distance (ignore Y)
            let current_2d = Vec3::new(transform.translation.x, 0.0, transform.translation.z);
            let target_2d = Vec3::new(target.x, 0.0, target.z);
            let direction = (target_2d - current_2d).normalize_or_zero();
            let distance = current_2d.distance(target_2d);

            if distance > 0.1 {
                // Calculate movement
                let move_distance = player.speed * time.delta_secs();
                let movement = direction * move_distance.min(distance);

                // Apply movement to controller
                controller.translation = Some(movement);

                println!(
                    "Movement: direction=({:.2}, {:.2}, {:.2}) distance={:.2} movement=({:.3}, {:.3}, {:.3})",
                    direction.x,
                    direction.y,
                    direction.z,
                    distance,
                    movement.x,
                    movement.y,
                    movement.z
                );
            } else {
                // Reached target
                player.target = None;
                controller.translation = Some(Vec3::ZERO);
                println!("Target reached, stopping movement");
            }
        } else {
            // No target, no movement
            controller.translation = Some(Vec3::ZERO);
        }
    }
}

fn log_state(
    query: Query<(
        Entity,
        &Transform,
        &SimplePlayer,
        &KinematicCharacterController,
        Option<&KinematicCharacterControllerOutput>,
    )>,
) {
    for (entity, transform, player, controller, output) in query.iter() {
        let controller_move = controller.translation.unwrap_or(Vec3::ZERO);

        if player.target.is_some() || controller_move.length() > 0.001 {
            let output_str = output
                .map(|o| {
                    format!(
                        "effective=({:.3}, {:.3}, {:.3}) grounded={}",
                        o.effective_translation.x,
                        o.effective_translation.y,
                        o.effective_translation.z,
                        o.grounded
                    )
                })
                .unwrap_or_else(|| "None".to_string());

            println!(
                "State: entity={:?} pos=({:.3}, {:.3}, {:.3}) controller_move=({:.3}, {:.3}, {:.3}) output={}",
                entity,
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
                controller_move.x,
                controller_move.y,
                controller_move.z,
                output_str
            );
        }
    }
}
