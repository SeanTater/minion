use bevy::prelude::*;
use crate::components::{Enemy, Name as EnemyName};
use crate::resources::GameState;

pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_enemy_tooltips,
            position_tooltips,
            update_tooltip_resources,
            cleanup_dead_enemy_tooltips,
        ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct EnemyTooltip {
    pub enemy_entity: Entity,
}

#[derive(Component)]
pub struct TooltipHealthBar {
    pub enemy_entity: Entity,
}

#[derive(Component)]
pub struct TooltipManaBar {
    pub enemy_entity: Entity,
}

#[derive(Component)]
pub struct TooltipEnergyBar {
    pub enemy_entity: Entity,
}

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
            commands.spawn((
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
            )).with_children(|parent| {
                // Enemy name
                parent.spawn((
                    Text::new(name.0.clone()),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..default()
                    },
                    TooltipNameText,
                ));
                
                // Health bar container
                parent.spawn(Node {
                    width: Val::Px(100.0),
                    height: Val::Px(6.0),
                    margin: UiRect::vertical(Val::Px(1.0)),
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
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                        TooltipHealthBar { enemy_entity },
                    ));
                });

                // Mana bar container
                parent.spawn(Node {
                    width: Val::Px(100.0),
                    height: Val::Px(6.0),
                    margin: UiRect::vertical(Val::Px(1.0)),
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
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.8)),
                        TooltipManaBar { enemy_entity },
                    ));
                });

                // Energy bar container
                parent.spawn(Node {
                    width: Val::Px(100.0),
                    height: Val::Px(6.0),
                    margin: UiRect::vertical(Val::Px(1.0)),
                    ..default()
                }).with_children(|energy_container| {
                    // Energy bar background
                    energy_container.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ));
                    
                    // Energy bar fill
                    energy_container.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.8, 0.8, 0.2)),
                        TooltipEnergyBar { enemy_entity },
                    ));
                });
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
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };
    let Ok(_window) = windows.single() else { return; };
    
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
    enemy_query: Query<&Enemy>,
    mut health_bar_query: Query<(&mut Node, &TooltipHealthBar), (With<TooltipHealthBar>, Without<TooltipManaBar>, Without<TooltipEnergyBar>)>,
    mut mana_bar_query: Query<(&mut Node, &TooltipManaBar), (With<TooltipManaBar>, Without<TooltipHealthBar>, Without<TooltipEnergyBar>)>,
    mut energy_bar_query: Query<(&mut Node, &TooltipEnergyBar), (With<TooltipEnergyBar>, Without<TooltipHealthBar>, Without<TooltipManaBar>)>,
) {
    // Update health bars
    for (mut node, health_bar) in health_bar_query.iter_mut() {
        if let Ok(enemy) = enemy_query.get(health_bar.enemy_entity) {
            let health_percent = enemy.health.percentage();
            node.width = Val::Percent(health_percent * 100.0);
        }
    }
    
    // Update mana bars
    for (mut node, mana_bar) in mana_bar_query.iter_mut() {
        if let Ok(enemy) = enemy_query.get(mana_bar.enemy_entity) {
            let mana_percent = enemy.mana.percentage();
            node.width = Val::Percent(mana_percent * 100.0);
        }
    }
    
    // Update energy bars
    for (mut node, energy_bar) in energy_bar_query.iter_mut() {
        if let Ok(enemy) = enemy_query.get(energy_bar.enemy_entity) {
            let energy_percent = enemy.energy.percentage();
            node.width = Val::Percent(energy_percent * 100.0);
        }
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