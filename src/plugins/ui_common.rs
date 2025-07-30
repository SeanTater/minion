use bevy::prelude::*;
use bevy::app::AppExit;
use crate::resources::GameState;

pub fn handle_exit_events(
    keys: Res<ButtonInput<KeyCode>>, 
    mut exit: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => {
                // From game, go back to main menu
                next_state.set(GameState::MainMenu);
            }
            GameState::Settings => {
                // From settings, go back to main menu
                next_state.set(GameState::MainMenu);
            }
            GameState::MainMenu => {
                // From main menu, exit the game
                exit.write(AppExit::Success);
            }
        }
    }
}