use crate::components::{ResourceDisplay, ResourceType, *};
use crate::config::load_config_or_default;
use crate::plugins::ui_common::{handle_exit_events, spawn_resource_bar, update_resource_display};
use crate::resources::*;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(GameConfig::default())
            .add_systems(Startup, load_game_config)
            .add_systems(Update, handle_exit_events)
            .add_systems(OnEnter(GameState::Playing), setup_game_ui_simple)
            .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), cleanup_game_ui);
    }
}

#[derive(Component)]
pub struct GameUI;

// Old resource bar components removed - now using unified ResourceDisplay

fn load_game_config(mut commands: Commands) {
    let config = load_config_or_default();
    commands.insert_resource(config);
}

fn setup_game_ui_simple(mut commands: Commands) {
    // HUD container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GameUI,
        ))
        .with_children(|parent| {
            // Top HUD bar
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|hud| {
                    // Get player entity for resource displays (we'll find it in the update system)
                    let temp_entity = Entity::PLACEHOLDER; // Will be updated in the system

                    // Create consolidated resource bars
                    for (i, resource_type) in [
                        ResourceType::Health,
                        ResourceType::Mana,
                        ResourceType::Energy,
                    ]
                    .iter()
                    .enumerate()
                    {
                        spawn_resource_bar(
                            hud,
                            *resource_type,
                            temp_entity,
                            200.0,
                            25.0,
                            if i < 2 {
                                UiRect::right(Val::Px(15.0))
                            } else {
                                UiRect::ZERO
                            },
                            true,
                        );
                    }
                });

            // Controls text at bottom
            parent.spawn((
                Text::new("Controls: Left Click=Move, Right Click=Shoot, Space=Area Effect"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(10.0),
                    bottom: Val::Px(10.0),
                    ..default()
                },
            ));
        });
}

fn update_hud(
    player_query: Query<(Entity, &Player)>,
    player_only_query: Query<&Player>, // For shared function
    enemy_query: Query<&Enemy>,        // Added for shared function
    mut resource_displays: Query<(Entity, &mut ResourceDisplay), With<ResourceDisplay>>,
    mut bar_fills: Query<&mut Node, (Without<ResourceDisplay>, Without<Text>)>,
    mut texts: Query<&mut Text, Without<ResourceDisplay>>,
    children_query: Query<&Children>,
) {
    let (player_entity, _player) = player_query
        .single()
        .expect("Player should always exist when in Playing state");
    // Update resource displays to point to the correct player entity and update their children
    for (display_entity, mut display) in resource_displays.iter_mut() {
        if display.target_entity == Entity::PLACEHOLDER {
            display.target_entity = player_entity;
        }

        if display.target_entity == player_entity {
            // Use shared resource display update function
            update_resource_display(
                &display,
                &player_only_query,
                &enemy_query,
                display_entity,
                &mut bar_fills,
                &mut texts,
                &children_query,
            );
        }
    }
}

fn cleanup_game_ui(mut commands: Commands, game_ui_query: Query<Entity, With<GameUI>>) {
    // Remove all game UI elements when exiting the Playing state
    for ui_entity in game_ui_query.iter() {
        commands.entity(ui_entity).despawn();
    }
}
