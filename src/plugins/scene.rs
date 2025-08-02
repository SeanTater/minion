use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::components::*;
use crate::resources::GameState;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), setup_scene)
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
) {
    // Only spawn ground if it doesn't exist
    if ground_query.is_empty() {
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 0.3),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(10.0, 0.1, 10.0), // 20x20 ground plane with small thickness
            Ground,
        ));
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
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(7.0, 12.0, 7.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            CameraFollow {
                offset: Vec3::new(7.0, 12.0, 7.0),
            },
        ));
    }
}

fn follow_camera(
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    mut camera_query: Query<(&mut Transform, &CameraFollow), Without<Player>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut camera_transform, follow) in camera_query.iter_mut() {
            let target_pos = player_transform.translation + follow.offset;
            camera_transform.translation = target_pos;
            camera_transform.look_at(player_transform.translation, Vec3::Y);
        }
    }
}