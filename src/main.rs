use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use minion::plugins::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Minion - Diablo-like Game".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins((
            EguiUiPlugin,
            UiPlugin, // Re-enabled for camera setup
            MapLoaderPlugin,
            EnvironmentPlugin, // Must come after MapLoaderPlugin to access MapDefinition resource
            ScenePlugin,
            PlayerPlugin,
            EnemyPlugin,
            CombatPlugin,
            TooltipPlugin,
        ))
        .run();
}
