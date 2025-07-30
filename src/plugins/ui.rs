use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::window::WindowCloseRequested;
use bevy::input::keyboard::KeyboardInput;
use crate::resources::*;
use crate::config::{load_config, save_config};
use crate::components::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .insert_resource(GameConfig::default())
            .insert_resource(UsernameInput::default())
            .add_systems(Startup, (load_game_config, setup_simple_ui))
            .add_systems(Update, (
                handle_simple_input,
                handle_exit_events,
                handle_username_input,
            ))
            .add_systems(OnEnter(GameState::Playing), (cleanup_main_menu, setup_game_ui_simple))
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

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct HealthText;

#[derive(Component)]
pub struct ManaBar;

#[derive(Component)]
pub struct ManaText;

fn load_game_config(mut commands: Commands, mut username_input: ResMut<UsernameInput>) {
    let config = load_config();
    
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
    commands.spawn((
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
    )).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("MINION"),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
        
        // Username label
        parent.spawn((
            Text::new("Username:"),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::top(Val::Px(30.0)),
                ..default()
            },
        ));
        
        // Username input box
        parent.spawn((
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
        )).with_children(|input_parent| {
            input_parent.spawn((
                Text::new(if game_config.username.is_empty() { 
                    "Enter your name..." 
                } else { 
                    &game_config.username 
                }),
                TextFont { font_size: 18.0, ..default() },
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
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ));
        
        // Exit instruction
        parent.spawn((
            Text::new("Press Escape to exit"),
            TextFont { font_size: 14.0, ..default() },
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
            let _ = save_config(&game_config);
        }
        next_state.set(GameState::Playing);
    }
}

fn handle_exit_events(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn handle_window_close(
    mut window_close_events: EventReader<WindowCloseRequested>,
    mut exit: EventWriter<AppExit>,
) {
    for _event in window_close_events.read() {
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
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        GameUI,
    )).with_children(|parent| {
        // Top HUD bar
        parent.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(80.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        }).with_children(|hud| {
            // Health bar container
            hud.spawn(Node {
                width: Val::Px(200.0),
                height: Val::Px(25.0),
                margin: UiRect::right(Val::Px(15.0)),
                ..default()
            }).with_children(|health_container| {
                // Health bar background
                health_container.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ));
                
                // Health bar fill
                health_container.spawn((
                    Node {
                        width: Val::Percent(100.0), // Will be updated dynamically
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                    HealthBar,
                ));
                
                // Health text
                health_container.spawn((
                    Text::new("HP: 100/100"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(5.0),
                        top: Val::Px(2.0),
                        ..default()
                    },
                    HealthText,
                ));
            });
            
            // Mana bar container
            hud.spawn(Node {
                width: Val::Px(200.0),
                height: Val::Px(25.0),
                ..default()
            }).with_children(|mana_container| {
                // Mana bar background
                mana_container.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ));
                
                // Mana bar fill
                mana_container.spawn((
                    Node {
                        width: Val::Percent(100.0), // Will be updated dynamically
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
                    ManaBar,
                ));
                
                // Mana text
                mana_container.spawn((
                    Text::new("MP: 50/50"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(5.0),
                        top: Val::Px(2.0),
                        ..default()
                    },
                    ManaText,
                ));
            });
        });
        
        // Controls text at bottom
        parent.spawn((
            Text::new("Controls: Left Click=Move, Right Click=Shoot, Space=Area Effect"),
            TextFont { font_size: 16.0, ..default() },
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
    player_query: Query<&Player>,
    mut health_bar_query: Query<&mut Node, (With<HealthBar>, Without<ManaBar>)>,
    mut mana_bar_query: Query<&mut Node, (With<ManaBar>, Without<HealthBar>)>,
    mut health_text_query: Query<&mut Text, (With<HealthText>, Without<ManaText>)>,
    mut mana_text_query: Query<&mut Text, (With<ManaText>, Without<HealthText>)>,
) {
    if let Ok(player) = player_query.single() {
        // Update health bar
        if let Ok(mut health_bar) = health_bar_query.single_mut() {
            let health_percent = player.health.percentage();
            health_bar.width = Val::Percent(health_percent * 100.0);
        }
        
        // Update mana bar
        if let Ok(mut mana_bar) = mana_bar_query.single_mut() {
            let mana_percent = player.mana.percentage();
            mana_bar.width = Val::Percent(mana_percent * 100.0);
        }
        
        // Update health text
        if let Ok(mut health_text) = health_text_query.single_mut() {
            **health_text = format!("HP: {}", player.health);
        }
        
        // Update mana text
        if let Ok(mut mana_text) = mana_text_query.single_mut() {
            **mana_text = format!("MP: {}", player.mana);
        }
    }
}

fn handle_username_input(
    mut username_input: ResMut<UsernameInput>,
    mut key_events: EventReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    username_display_query: Query<&Children, With<UsernameDisplay>>,
    mut text_query: Query<&mut Text>,
    game_config: Res<GameConfig>,
) {
    // Handle character input via keyboard events
    for event in key_events.read() {
        if event.state.is_pressed() {
            match event.key_code {
                    KeyCode::KeyA => username_input.text.push('a'),
                    KeyCode::KeyB => username_input.text.push('b'),
                    KeyCode::KeyC => username_input.text.push('c'),
                    KeyCode::KeyD => username_input.text.push('d'),
                    KeyCode::KeyE => username_input.text.push('e'),
                    KeyCode::KeyF => username_input.text.push('f'),
                    KeyCode::KeyG => username_input.text.push('g'),
                    KeyCode::KeyH => username_input.text.push('h'),
                    KeyCode::KeyI => username_input.text.push('i'),
                    KeyCode::KeyJ => username_input.text.push('j'),
                    KeyCode::KeyK => username_input.text.push('k'),
                    KeyCode::KeyL => username_input.text.push('l'),
                    KeyCode::KeyM => username_input.text.push('m'),
                    KeyCode::KeyN => username_input.text.push('n'),
                    KeyCode::KeyO => username_input.text.push('o'),
                    KeyCode::KeyP => username_input.text.push('p'),
                    KeyCode::KeyQ => username_input.text.push('q'),
                    KeyCode::KeyR => username_input.text.push('r'),
                    KeyCode::KeyS => username_input.text.push('s'),
                    KeyCode::KeyT => username_input.text.push('t'),
                    KeyCode::KeyU => username_input.text.push('u'),
                    KeyCode::KeyV => username_input.text.push('v'),
                    KeyCode::KeyW => username_input.text.push('w'),
                    KeyCode::KeyX => username_input.text.push('x'),
                    KeyCode::KeyY => username_input.text.push('y'),
                    KeyCode::KeyZ => username_input.text.push('z'),
                    KeyCode::Space => username_input.text.push(' '),
                    KeyCode::Digit1 => username_input.text.push('1'),
                    KeyCode::Digit2 => username_input.text.push('2'),
                    KeyCode::Digit3 => username_input.text.push('3'),
                    KeyCode::Digit4 => username_input.text.push('4'),
                    KeyCode::Digit5 => username_input.text.push('5'),
                    KeyCode::Digit6 => username_input.text.push('6'),
                    KeyCode::Digit7 => username_input.text.push('7'),
                    KeyCode::Digit8 => username_input.text.push('8'),
                    KeyCode::Digit9 => username_input.text.push('9'),
                    KeyCode::Digit0 => username_input.text.push('0'),
                    _ => {}
                }
                
            // Limit username length
            if username_input.text.len() > 20 {
                username_input.text.truncate(20);
            }
        }
    }
    
    // Handle backspace
    if keys.just_pressed(KeyCode::Backspace) {
        username_input.text.pop();
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