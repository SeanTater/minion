use bevy::prelude::*;
use crate::components::*;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_scene)
            .add_systems(Update, follow_camera);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(20.0, 20.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 0.3),
                ..default()
            }),
            ..default()
        },
        Ground,
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
    });

    // Camera with isometric view that follows player
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 15.0, 10.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraFollow {
            offset: Vec3::new(10.0, 15.0, 10.0),
        },
    ));
}

fn follow_camera(
    player_query: Query<&Transform, (With<Player>, Without<CameraFollow>)>,
    mut camera_query: Query<(&mut Transform, &CameraFollow), Without<Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut camera_transform, follow) in camera_query.iter_mut() {
            let target_pos = player_transform.translation + follow.offset;
            camera_transform.translation = target_pos;
            camera_transform.look_at(player_transform.translation, Vec3::Y);
        }
    }
}