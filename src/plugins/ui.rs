use crate::components::{*, ResourceType, ResourceDisplay};
use crate::config::{load_config_or_default, save_config};
use crate::resources::*;
use bevy::app::AppExit;
use bevy::input::keyboard::{KeyboardInput, Key};
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(GameConfig::default())
            .insert_resource(UsernameInput::default())
            .add_systems(Startup, load_game_config)
            .add_systems(
                Update,
                handle_exit_events,
            )
            .add_systems(
                OnEnter(GameState::Playing),
                setup_game_ui_simple,
            )
            .add_systems(Update, update_hud.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource, Default)]
pub struct UsernameInput {
    pub text: String,
}

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub struct GameUI;

#[derive(Component)]
pub struct UsernameDisplay;

#[derive(Component)]
pub struct StartButton;

#[derive(Component)]
pub struct ScoreText;

// Old resource bar components removed - now using unified ResourceDisplay

fn load_game_config(mut commands: Commands, mut username_input: ResMut<UsernameInput>) {
    let config = load_config_or_default();

    // Initialize username input with saved username
    username_input.text = config.username.clone();

    commands.insert_resource(config);
}

fn setup_simple_ui(mut commands: Commands, game_config: Res<GameConfig>) {
    // Create a simple camera for now until we get bevy_lunex working
    commands.spawn((
        Camera2d,
        Camera {
            order: 1, // Ensure UI camera renders after 3D camera
            ..default()
        },
    ));

    // Add a simple background color
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            MainMenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("MINION"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Username label
            parent.spawn((
                Text::new("Username:"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Username input box
            parent
                .spawn((
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(40.0),
                        margin: UiRect::all(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                    UsernameDisplay,
                ))
                .with_children(|input_parent| {
                    input_parent.spawn((
                        Text::new(if game_config.username.is_empty() {
                            "Enter your name..."
                        } else {
                            &game_config.username
                        }),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(if game_config.username.is_empty() {
                            Color::srgb(0.5, 0.5, 0.5)
                        } else {
                            Color::WHITE
                        }),
                    ));
                });

            // Instructions
            parent.spawn((
                Text::new("Type to enter name, Press Enter to play"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::all(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Exit instruction
            parent.spawn((
                Text::new("Press Escape to exit"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
}

fn handle_simple_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_config: ResMut<GameConfig>,
    username_input: Res<UsernameInput>,
) {
    // Press Enter to start the game
    if keys.just_pressed(KeyCode::Enter) {
        // Save username if it was entered
        if !username_input.text.trim().is_empty() {
            game_config.username = username_input.text.trim().to_string();
            if let Err(err) = save_config(&game_config) {
                eprintln!("Warning: Failed to save config: {}", err);
            }
        }
        next_state.set(GameState::Playing);
    }
}

fn handle_exit_events(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn cleanup_main_menu(
    mut commands: Commands,
    menu_query: Query<Entity, With<MainMenuUI>>,
    camera_query: Query<Entity, With<Camera2d>>,
) {
    // Remove main menu UI
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove 2D camera to avoid conflicts with 3D camera
    for entity in camera_query.iter() {
        commands.entity(entity).despawn();
    }
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
                    for (i, resource_type) in [ResourceType::Health, ResourceType::Mana, ResourceType::Energy].iter().enumerate() {
                        spawn_resource_bar(hud, *resource_type, temp_entity, i < 2); // Add margin to first two
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

fn spawn_resource_bar(
    parent: &mut bevy::prelude::ChildSpawnerCommands,
    resource_type: ResourceType,
    target_entity: Entity,
    add_margin: bool,
) {
    parent.spawn((
        Node {
            width: Val::Px(200.0),
            height: Val::Px(25.0),
            margin: if add_margin { UiRect::right(Val::Px(15.0)) } else { UiRect::ZERO },
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        ResourceDisplay::new(resource_type, target_entity, true),
    )).with_children(|bar_container| {
        // Resource bar fill
        bar_container.spawn((
            Node {
                width: Val::Percent(100.0), // Will be updated dynamically
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(resource_type.color()),
        ));
        
        // Resource text overlay
        bar_container.spawn((
            Text::new(format!("{}: 100/100", resource_type.label())),
            TextFont {
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(5.0),
                top: Val::Px(2.0),
                ..default()
            },
        ));
    });
}

fn update_hud(
    player_query: Query<(Entity, &Player)>,
    mut resource_displays: Query<(Entity, &mut ResourceDisplay), With<ResourceDisplay>>,
    mut bar_fills: Query<&mut Node, (Without<ResourceDisplay>, Without<Text>)>,
    mut texts: Query<&mut Text, Without<ResourceDisplay>>,
    children_query: Query<&Children>,
) {
    if let Ok((player_entity, player)) = player_query.single() {
        // Update resource displays to point to the correct player entity and update their children
        for (display_entity, mut display) in resource_displays.iter_mut() {
            if display.target_entity == Entity::PLACEHOLDER {
                display.target_entity = player_entity;
            }
            
            if display.target_entity == player_entity {
                let (current, max) = match display.resource_type {
                    ResourceType::Health => (player.health.current, player.health.max),
                    ResourceType::Mana => (player.mana.current, player.mana.max),
                    ResourceType::Energy => (player.energy.current, player.energy.max),
                };
                
                // Update the bar fill and text children of this resource display
                if let Ok(children) = children_query.get(display_entity) {
                    for child_entity in children.iter() {
                        // Update bar fill (first child)
                        if let Ok(mut bar_node) = bar_fills.get_mut(child_entity) {
                            let percentage = if max > 0.0 { current / max } else { 0.0 };
                            bar_node.width = Val::Percent(percentage * 100.0);
                        }
                        
                        // Update text (second child)
                        if let Ok(mut text) = texts.get_mut(child_entity) {
                            **text = format!("{}: {:.0}/{:.0}", display.resource_type.label(), current, max);
                        }
                    }
                }
            }
        }
    }
}

fn handle_username_input(
    mut username_input: ResMut<UsernameInput>,
    mut keyboard_events: EventReader<KeyboardInput>,
    username_display_query: Query<&Children, With<UsernameDisplay>>,
    mut text_query: Query<&mut Text>,
    game_config: Res<GameConfig>,
) {
    // Handle keyboard input events
    for event in keyboard_events.read() {
        // Only process key presses, not releases
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Character(character) => {
                // Allow alphanumeric characters, spaces, underscores, and hyphens
                for ch in character.chars() {
                    if ch.is_alphanumeric() || ch == ' ' || ch == '_' || ch == '-' {
                        username_input.text.push(ch);
                    }
                }
                
                // Limit username length
                if username_input.text.len() > 20 {
                    username_input.text.truncate(20);
                }
            }
            Key::Space => {
                username_input.text.push(' ');
                if username_input.text.len() > 20 {
                    username_input.text.truncate(20);
                }
            }
            Key::Backspace => {
                username_input.text.pop();
            }
            _ => {}
        }
    }

    // Update display text
    if let Ok(children) = username_display_query.single() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                if username_input.text.is_empty() {
                    if game_config.username.is_empty() {
                        **text = "Enter your name...".to_string();
                        // Could set color to gray here if needed
                    } else {
                        **text = game_config.username.clone();
                    }
                } else {
                    **text = username_input.text.clone();
                }
            }
        }
    }
}
