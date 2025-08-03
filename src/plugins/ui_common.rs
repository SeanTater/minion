use crate::components::{Enemy, Player, ResourceDisplay, ResourceType};
use crate::resources::GameState;
use bevy::app::AppExit;
use bevy::prelude::*;

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

/// Shared resource display update function
/// Updates resource bars for both player HUD and enemy tooltips
pub fn update_resource_display(
    display: &ResourceDisplay,
    player_query: &Query<&Player>,
    enemy_query: &Query<&Enemy>,
    display_entity: Entity,
    bar_fills: &mut Query<&mut Node, (Without<ResourceDisplay>, Without<Text>)>,
    texts: &mut Query<&mut Text, Without<ResourceDisplay>>,
    children_query: &Query<&Children>,
) {
    // Get the resource values based on the target entity type
    let (current, max) = if let Ok(player) = player_query.get(display.target_entity) {
        // Target is a player
        match display.resource_type {
            ResourceType::Health => (player.health.current, player.health.max),
            ResourceType::Mana => (player.mana.current, player.mana.max),
            ResourceType::Energy => (player.energy.current, player.energy.max),
        }
    } else if let Ok(enemy) = enemy_query.get(display.target_entity) {
        // Target is an enemy
        match display.resource_type {
            ResourceType::Health => (enemy.health.current, enemy.health.max),
            ResourceType::Mana => (enemy.mana.current, enemy.mana.max),
            ResourceType::Energy => (enemy.energy.current, enemy.energy.max),
        }
    } else {
        // Target entity doesn't exist or doesn't have resources
        return;
    };

    // Update the bar fill and text children of this resource display
    if let Ok(children) = children_query.get(display_entity) {
        for child_entity in children.iter() {
            // Update bar fill
            if let Ok(mut bar_node) = bar_fills.get_mut(child_entity) {
                let percentage = if max > 0.0 { current / max } else { 0.0 };
                bar_node.width = Val::Percent(percentage * 100.0);
            }

            // Update text (only if the display should show text)
            if display.show_text {
                if let Ok(mut text) = texts.get_mut(child_entity) {
                    **text = format!(
                        "{}: {:.0}/{:.0}",
                        display.resource_type.label(),
                        current,
                        max
                    );
                }
            }
        }
    }
}

/// Unified resource bar spawning function for both HUD and tooltips
pub fn spawn_resource_bar(
    parent: &mut bevy::prelude::ChildSpawnerCommands,
    resource_type: ResourceType,
    target_entity: Entity,
    width: f32,
    height: f32,
    margin: UiRect,
    show_text: bool,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(width),
                height: Val::Px(height),
                margin,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ResourceDisplay::new(resource_type, target_entity, show_text),
        ))
        .with_children(|bar_container| {
            // Resource bar fill
            bar_container.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(resource_type.color()),
            ));

            // Resource text overlay (only for HUD bars)
            if show_text {
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
            }
        });
}
