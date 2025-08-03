use crate::components::*;
use crate::map::MapDefinition;
use crate::resources::GameState;
use crate::terrain::generate_terrain_mesh_and_collider;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            setup_scene.after(crate::plugins::map_loader::load_map),
        )
        .add_systems(Update, follow_camera.run_if(in_state(GameState::Playing)));
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_query: Query<&Camera3d>,
    ground_query: Query<&Ground>,
    light_query: Query<&SceneLight>,
    map: Res<MapDefinition>,
) {
    // Only spawn ground if it doesn't exist
    if ground_query.is_empty() {
        // Generate real 3D terrain from heightmap data
        match generate_terrain_mesh_and_collider(&map.terrain) {
            Ok((mesh, collider)) => {
                let terrain_width = map.terrain.width as f32 * map.terrain.scale;
                let terrain_height = map.terrain.height as f32 * map.terrain.scale;
                let center_x_offset = terrain_width / 2.0;
                let center_z_offset = terrain_height / 2.0;
                info!(
                    "Generated terrain: {}x{} (scale: {}), world bounds: ({:.1}, {:.1}) to ({:.1}, {:.1})",
                    map.terrain.width,
                    map.terrain.height,
                    map.terrain.scale,
                    -center_x_offset,
                    -center_z_offset,
                    center_x_offset - map.terrain.scale,
                    center_z_offset - map.terrain.scale
                );

                commands.spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.3, 0.5, 0.3),
                        ..default()
                    })),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    RigidBody::Fixed,
                    collider,
                    Ground,
                ));
            }
            Err(e) => {
                warn!("Failed to generate terrain mesh: {e}");

                // Fallback to flat terrain if heightmap generation fails
                let terrain_width = map.terrain.width as f32 * map.terrain.scale;
                let terrain_height = map.terrain.height as f32 * map.terrain.scale;

                commands.spawn((
                    Mesh3d(
                        meshes.add(
                            Plane3d::default()
                                .mesh()
                                .size(terrain_width, terrain_height),
                        ),
                    ),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.3, 0.5, 0.3),
                        ..default()
                    })),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    RigidBody::Fixed,
                    Collider::cuboid(terrain_width / 2.0, 0.1, terrain_height / 2.0),
                    Ground,
                ));
            }
        }
    }

    // Only spawn light if it doesn't exist
    if light_query.is_empty() {
        commands.spawn((
            DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            Transform {
                translation: Vec3::new(0.0, 2.0, 0.0),
                rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                ..default()
            },
            SceneLight,
        ));

        // Ambient light (resource - only set once)
        commands.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 300.0,
            affects_lightmapped_meshes: false,
        });
    }

    // Only spawn camera if it doesn't exist
    if camera_query.is_empty() {
        info!(
            "Spawning camera at: ({}, {}, {}), looking at origin",
            7.0, 12.0, 7.0
        );

        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(7.0, 12.0, 7.0).looking_at(Vec3::ZERO, Vec3::Y),
            CameraFollow {
                offset: Vec3::new(14.0, 24.0, 14.0),
            },
        ));
    }
}

fn follow_camera(
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    mut camera_query: Query<(&mut Transform, &CameraFollow), Without<Player>>,
) {
    let player_transform = player_query.single()
        .expect("Player should always exist when in Playing state");
    for (mut camera_transform, follow) in camera_query.iter_mut() {
        let target_pos = player_transform.translation + follow.offset;
        camera_transform.translation = target_pos;
        camera_transform.look_at(player_transform.translation, Vec3::Y);
    }
}
