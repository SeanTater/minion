use crate::map::{EnvironmentObject, MapDefinition};
use crate::resources::GameState;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing), 
            spawn_environment_objects.after(crate::plugins::map_loader::load_map)
        );
    }
}

/// Marker component for environment objects
#[derive(Component)]
pub struct EnvironmentObjectMarker {
    pub object_type: String,
}

fn spawn_environment_objects(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    map: Res<MapDefinition>,
) {
    println!(
        "Spawning {} environment objects...",
        map.environment_objects.len()
    );

    for obj in &map.environment_objects {
        spawn_single_environment_object(&mut commands, &mut meshes, &mut materials, obj);
    }
}

fn spawn_single_environment_object(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    obj: &EnvironmentObject,
) {
    // Create transform from object data
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        obj.rotation.x,
        obj.rotation.y,
        obj.rotation.z,
    );
    let transform = Transform {
        translation: obj.position,
        rotation,
        scale: obj.scale,
    };

    // Choose collider based on object type
    let collider = get_object_collider(&obj.object_type, &obj.scale);

    // Create primitive mesh and material based on object type
    let (mesh, material) =
        create_placeholder_mesh_and_material(&obj.object_type, meshes, materials);

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        collider,
        RigidBody::Fixed,
        EnvironmentObjectMarker {
            object_type: obj.object_type.clone(),
        },
        Name::new(format!("EnvObject_{}", obj.object_type)),
    ));
}

fn create_placeholder_mesh_and_material(
    object_type: &str,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> (Handle<Mesh>, Handle<StandardMaterial>) {
    let (mesh, color) = match object_type {
        "tree" => {
            // Tree: tall cylinder (trunk-like)
            (
                Mesh::from(Cylinder::new(0.2, 2.0)),
                Color::srgb(0.4, 0.2, 0.0),
            )
        }
        "rock" => {
            // Rock: slightly flattened sphere
            (
                Sphere::new(0.8).mesh().uv(8, 6),
                Color::srgb(0.5, 0.5, 0.5),
            )
        }
        "boulder" => {
            // Boulder: larger, more irregular (use a cube for now)
            (
                Mesh::from(Cuboid::new(1.2, 1.0, 1.1)),
                Color::srgb(0.4, 0.4, 0.45),
            )
        }
        "grass" => {
            // Grass: small flat cylinder
            (
                Mesh::from(Cylinder::new(0.3, 0.1)),
                Color::srgb(0.2, 0.8, 0.2),
            )
        }
        _ => {
            // Default: simple cube
            (
                Mesh::from(Cuboid::new(1.0, 1.0, 1.0)),
                Color::srgb(0.6, 0.3, 0.6),
            )
        }
    };

    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(StandardMaterial {
        base_color: color,
        ..default()
    });

    (mesh_handle, material_handle)
}

fn get_object_collider(object_type: &str, scale: &Vec3) -> Collider {
    match object_type {
        "tree" => {
            // Tree: tall cylinder
            Collider::cylinder(scale.y * 1.5, scale.x * 0.3)
        }
        "rock" | "boulder" => {
            // Rock/boulder: roughly spherical
            Collider::ball(scale.x * 0.5)
        }
        "grass" => {
            // Grass: very small collider or no collider
            Collider::ball(scale.x * 0.1)
        }
        _ => {
            // Default: box collider
            Collider::cuboid(scale.x * 0.5, scale.y * 0.5, scale.z * 0.5)
        }
    }
}