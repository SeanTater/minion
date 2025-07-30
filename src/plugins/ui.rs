use bevy::prelude::*;
use bevy::app::AppExit;
use bevy::window::WindowCloseRequested;
use bevy::input::keyboard::{KeyboardInput};
use bevy::input::ButtonState;
use crate::resources::*;
use crate::config::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<GameState>()
            .insert_resource(GameConfig::default())
            .insert_resource(UsernameInput::default())
            .add_systems(Startup, (load_game_config, setup_ui_camera, setup_main_menu))
            .add_systems(Update, (
                handle_username_input,
                handle_start_button,
                handle_exit_events,
            ).run_if(in_state(GameState::MainMenu)))
            .add_systems(Update, (
                handle_window_close,
                update_score_display,
            ))
            .add_systems(OnExit(GameState::MainMenu), (cleanup_main_menu, cleanup_ui_camera))
            .add_systems(OnEnter(GameState::Playing), setup_game_ui);
    }
}

#[derive(Resource, Default)]
pub struct UsernameInput {
    pub text: String,
}

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub struct UsernameTextInput;

#[derive(Component)]
pub struct StartButton;

#[derive(Component)]
pub struct GameUI;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct UsernameDisplay;

#[derive(Component)]
pub struct UiCamera;

fn load_game_config(mut commands: Commands) {
    let config = load_config();
    commands.insert_resource(config);
}

fn setup_ui_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), UiCamera));
}

fn setup_main_menu(mut commands: Commands) {
    // Main menu container
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::srgb(0.1, 0.1, 0.15).into(),
                ..default()
            },
            MainMenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn(TextBundle::from_section(
                "MINION",
                TextStyle {
                    font_size: 80.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(50.0)),
                ..default()
            }));

            // Subtitle
            parent.spawn(TextBundle::from_section(
                "A Diablo-like Action RPG",
                TextStyle {
                    font_size: 24.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            }));

            // Username input label
            parent.spawn(TextBundle::from_section(
                "Enter your username:",
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgb(0.8, 0.8, 0.8),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            }));

            // Username input field
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        margin: UiRect::bottom(Val::Px(30.0)),
                        ..default()
                    },
                    border_color: Color::srgb(0.6, 0.6, 0.6).into(),
                    background_color: Color::srgb(0.2, 0.2, 0.25).into(),
                    ..default()
                },
                UsernameTextInput,
            )).with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ),
                    UsernameDisplay,
                ));
            });

            // Start button
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(50.0),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: Color::srgb(0.8, 0.8, 0.8).into(),
                    background_color: Color::srgb(0.3, 0.5, 0.8).into(),
                    ..default()
                },
                StartButton,
            )).with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "START",
                    TextStyle {
                        font_size: 24.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ));
            });

            // Instructions
            parent.spawn(TextBundle::from_section(
                "Controls: Right-click to shoot, Spacebar for area effect, Tab to cycle effects",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.6, 0.6, 0.6),
                    ..default()
                },
            ).with_style(Style {
                margin: UiRect::top(Val::Px(40.0)),
                ..default()
            }));
        });
}

fn handle_username_input(
    mut keyboard_events: EventReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut username_input: ResMut<UsernameInput>,
    mut text_query: Query<&mut Text, With<UsernameDisplay>>,
) {
    // Handle character input
    for event in keyboard_events.read() {
        if event.state == ButtonState::Pressed {
            if let Some(key_char) = match event.key_code {
                KeyCode::KeyA => Some('a'),
                KeyCode::KeyB => Some('b'),
                KeyCode::KeyC => Some('c'),
                KeyCode::KeyD => Some('d'),
                KeyCode::KeyE => Some('e'),
                KeyCode::KeyF => Some('f'),
                KeyCode::KeyG => Some('g'),
                KeyCode::KeyH => Some('h'),
                KeyCode::KeyI => Some('i'),
                KeyCode::KeyJ => Some('j'),
                KeyCode::KeyK => Some('k'),
                KeyCode::KeyL => Some('l'),
                KeyCode::KeyM => Some('m'),
                KeyCode::KeyN => Some('n'),
                KeyCode::KeyO => Some('o'),
                KeyCode::KeyP => Some('p'),
                KeyCode::KeyQ => Some('q'),
                KeyCode::KeyR => Some('r'),
                KeyCode::KeyS => Some('s'),
                KeyCode::KeyT => Some('t'),
                KeyCode::KeyU => Some('u'),
                KeyCode::KeyV => Some('v'),
                KeyCode::KeyW => Some('w'),
                KeyCode::KeyX => Some('x'),
                KeyCode::KeyY => Some('y'),
                KeyCode::KeyZ => Some('z'),
                KeyCode::Digit0 => Some('0'),
                KeyCode::Digit1 => Some('1'),
                KeyCode::Digit2 => Some('2'),
                KeyCode::Digit3 => Some('3'),
                KeyCode::Digit4 => Some('4'),
                KeyCode::Digit5 => Some('5'),
                KeyCode::Digit6 => Some('6'),
                KeyCode::Digit7 => Some('7'),
                KeyCode::Digit8 => Some('8'),
                KeyCode::Digit9 => Some('9'),
                KeyCode::Minus => Some('-'),
                _ => None,
            } {
                if username_input.text.len() < 20 {
                    username_input.text.push(key_char);
                }
            }
        }
    }

    // Handle backspace
    if keyboard.just_pressed(KeyCode::Backspace) {
        username_input.text.pop();
    }

    // Update text display
    if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = format!("{}|", username_input.text);
    }
}

fn handle_start_button(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartButton>),
    >,
    username_input: Res<UsernameInput>,
    mut game_config: ResMut<GameConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if !username_input.text.is_empty() {
                    game_config.username = username_input.text.clone();
                    let _ = save_config(&game_config);
                    next_state.set(GameState::Playing);
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.4, 0.6, 0.9).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.3, 0.5, 0.8).into();
            }
        }
    }
}

fn handle_exit_events(mut exit: EventWriter<AppExit>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn cleanup_ui_camera(mut commands: Commands, query: Query<Entity, With<UiCamera>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_game_ui(mut commands: Commands, game_config: Res<GameConfig>) {
    // In-game UI showing username and score
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(50.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    right: Val::Px(10.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            GameUI,
        ))
        .with_children(|parent| {
            // Username display
            parent.spawn(TextBundle::from_section(
                format!("Player: {}", game_config.username),
                TextStyle {
                    font_size: 20.0,
                    color: Color::srgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));

            // Score display
            parent.spawn((
                TextBundle::from_section(
                    format!("Score: {}", game_config.score),
                    TextStyle {
                        font_size: 20.0,
                        color: Color::srgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                ScoreText,
            ));
        });
}

fn handle_window_close(
    mut close_events: EventReader<WindowCloseRequested>,
    mut exit: EventWriter<AppExit>,
    game_config: Res<GameConfig>,
) {
    for _event in close_events.read() {
        let _ = save_config(&game_config);
        exit.send(AppExit::Success);
    }
}

fn update_score_display(
    game_config: Res<GameConfig>,
    mut score_query: Query<&mut Text, With<ScoreText>>,
) {
    if game_config.is_changed() {
        for mut text in score_query.iter_mut() {
            text.sections[0].value = format!("Score: {}", game_config.score);
        }
    }
}