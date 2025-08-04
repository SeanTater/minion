use crate::components::{Enemy, Name as EnemyName, Player, ResourceDisplay, ResourceType};
use crate::plugins::ui_common::{spawn_resource_bar, update_resource_display};
use crate::resources::GameState;
use bevy::prelude::*;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_enemy_tooltips,
                position_tooltips,
                update_tooltip_resources,
                cleanup_dead_enemy_tooltips,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_all_tooltips);
    }
}

#[derive(Component)]
pub struct EnemyTooltip {
    pub enemy_entity: Entity,
}

// Old tooltip resource bar components removed - now using unified ResourceDisplay

#[derive(Component)]
pub struct TooltipNameText;

fn update_enemy_tooltips(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Enemy, &EnemyName, &Transform)>,
    existing_tooltips: Query<&EnemyTooltip>,
) {
    // Create tooltips for enemies that don't have them
    for (enemy_entity, _enemy, name, _transform) in enemy_query.iter() {
        // Check if tooltip already exists
        let tooltip_exists = existing_tooltips
            .iter()
            .any(|tooltip| tooltip.enemy_entity == enemy_entity);

        if !tooltip_exists {
            // Create tooltip UI
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Px(120.0),
                        height: Val::Px(70.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                    EnemyTooltip { enemy_entity },
                    GlobalZIndex(1000), // Ensure tooltips appear on top
                ))
                .with_children(|parent| {
                    // Enemy name
                    parent.spawn((
                        Text::new(name.0.clone()),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                        TooltipNameText,
                    ));

                    // Create consolidated resource bars for tooltip
                    for resource_type in [
                        ResourceType::Health,
                        ResourceType::Mana,
                        ResourceType::Energy,
                    ] {
                        spawn_resource_bar(
                            parent,
                            resource_type,
                            enemy_entity,
                            100.0,
                            6.0,
                            UiRect::vertical(Val::Px(1.0)),
                            false,
                        );
                    }
                });
        }
    }
}

fn position_tooltips(
    mut tooltip_query: Query<(&mut Node, &EnemyTooltip)>,
    enemy_query: Query<(&Transform, &Enemy), With<Enemy>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    windows: Query<&Window>,
) {
    let (camera, camera_transform) = camera_query
        .single()
        .expect("Camera3d should always exist when game is running");
    let _window = windows
        .single()
        .expect("Primary window should always exist when game is running");

    for (mut tooltip_node, enemy_tooltip) in tooltip_query.iter_mut() {
        if let Ok((enemy_transform, enemy)) = enemy_query.get(enemy_tooltip.enemy_entity) {
            // Don't show tooltips for dying enemies
            if enemy.is_dying {
                tooltip_node.display = Display::None;
                continue;
            }

            tooltip_node.display = Display::Flex;

            // Convert world position to screen position
            let world_pos = enemy_transform.translation + Vec3::new(0.0, 2.0, 0.0); // Offset above enemy

            if let Ok(screen_pos) = camera.world_to_viewport(camera_transform, world_pos) {
                // Position tooltip at screen coordinates
                tooltip_node.left = Val::Px(screen_pos.x - 60.0); // Center horizontally
                tooltip_node.top = Val::Px(screen_pos.y - 70.0); // Position above enemy
            } else {
                // Hide tooltip if enemy is off-screen
                tooltip_node.display = Display::None;
            }
        } else {
            // Enemy no longer exists, hide tooltip
            tooltip_node.display = Display::None;
        }
    }
}

fn update_tooltip_resources(
    player_query: Query<&Player>, // Added for shared function
    enemy_query: Query<&Enemy>,
    resource_displays: Query<(Entity, &ResourceDisplay), With<ResourceDisplay>>,
    mut bar_fills: Query<&mut Node, (Without<ResourceDisplay>, Without<Text>)>,
    mut texts: Query<&mut Text, Without<ResourceDisplay>>, // Added for shared function
    children_query: Query<&Children>,
) {
    // Update each resource display using the shared function
    for (display_entity, display) in resource_displays.iter() {
        update_resource_display(
            display,
            &player_query,
            &enemy_query,
            display_entity,
            &mut bar_fills,
            &mut texts,
            &children_query,
        );
    }
}

fn cleanup_dead_enemy_tooltips(
    mut commands: Commands,
    tooltip_query: Query<(Entity, &EnemyTooltip)>,
    enemy_query: Query<&Enemy>,
) {
    for (tooltip_entity, enemy_tooltip) in tooltip_query.iter() {
        // If enemy no longer exists or is dying, remove tooltip
        if let Ok(enemy) = enemy_query.get(enemy_tooltip.enemy_entity) {
            if enemy.is_dying {
                commands.entity(tooltip_entity).despawn();
            }
        } else {
            // Enemy entity doesn't exist anymore
            commands.entity(tooltip_entity).despawn();
        }
    }
}

fn cleanup_all_tooltips(mut commands: Commands, tooltip_query: Query<Entity, With<EnemyTooltip>>) {
    // Remove all tooltips when exiting the Playing state
    for tooltip_entity in tooltip_query.iter() {
        commands.entity(tooltip_entity).despawn();
    }
}
