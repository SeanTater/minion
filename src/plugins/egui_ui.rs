use bevy::prelude::*;
use bevy::app::AppExit;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use crate::resources::{GameConfig, GameState, GameSettings};
use crate::config::save_config;

pub struct EguiUiPlugin;

impl Plugin for EguiUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_systems(Startup, setup_ui_camera)
            .add_systems(Update, simple_test_egui_system);
    }
}

fn setup_ui_camera(mut commands: Commands) {
    // Add UI camera with higher priority than 3D camera
    commands.spawn((
        Camera2d,
        Camera {
            order: 1, // Render after 3D camera
            ..default()
        },
    ));
}

fn handle_exit_events(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn simple_test_egui_system(mut contexts: EguiContexts) {
    println!("egui system running");
    if let Ok(ctx) = contexts.ctx_mut() {
        println!("got context");
        egui::CentralPanel::default().show(ctx, |ui| {
            println!("in central panel callback");
            ui.heading("It works!");
            ui.label("This is a test egui window");
            ui.separator();
            ui.label("If you can see this, egui is working");
        });
    }
}

fn main_menu_egui_system(
    mut contexts: EguiContexts,
    mut game_config: ResMut<GameConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Minion Main Menu")
            .default_size([400.0, 300.0])
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            
            // Title
            ui.heading("MINION");
            ui.add_space(30.0);
            
            // Username input
            ui.label("Username:");
            ui.text_edit_singleline(&mut game_config.username);
            ui.add_space(20.0);
            
            // Buttons
            ui.horizontal(|ui| {
                if ui.button("Play").clicked() {
                    if let Err(err) = save_config(&game_config) {
                        eprintln!("Warning: Failed to save config: {}", err);
                    }
                    next_state.set(GameState::Playing);
                }
                
                if ui.button("Settings").clicked() {
                    next_state.set(GameState::Settings);
                }
            });
            
                ui.add_space(20.0);
                ui.label("Press Escape to exit");
            });
        });
    }
}

fn settings_egui_system(
    mut contexts: EguiContexts,
    mut game_config: ResMut<GameConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::Window::new("Settings")
            .default_size([600.0, 500.0])
            .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.collapsing("Player Settings", |ui| {
                    ui.add(egui::Slider::new(&mut game_config.settings.player_movement_speed, 1.0..=10.0)
                        .text("Movement Speed"));
                    ui.add(egui::Slider::new(&mut game_config.settings.player_max_health, 50.0..=500.0)
                        .text("Max Health"));
                    ui.add(egui::Slider::new(&mut game_config.settings.player_max_mana, 25.0..=200.0)
                        .text("Max Mana"));
                    ui.add(egui::Slider::new(&mut game_config.settings.player_max_energy, 50.0..=300.0)
                        .text("Max Energy"));
                });

                ui.collapsing("Combat Settings", |ui| {
                    ui.add(egui::Slider::new(&mut game_config.settings.bullet_speed, 5.0..=30.0)
                        .text("Bullet Speed"));
                    ui.add(egui::Slider::new(&mut game_config.settings.bullet_damage, 0.5..=10.0)
                        .text("Bullet Damage"));
                    ui.add(egui::Slider::new(&mut game_config.settings.bullet_lifetime, 1.0..=10.0)
                        .text("Bullet Lifetime"));
                    ui.add(egui::Slider::new(&mut game_config.settings.magic_damage_per_second, 50.0..=300.0)
                        .text("Magic DPS"));
                    ui.add(egui::Slider::new(&mut game_config.settings.poison_damage_per_second, 30.0..=150.0)
                        .text("Poison DPS"));
                    ui.add(egui::Slider::new(&mut game_config.settings.magic_area_radius, 1.0..=8.0)
                        .text("Magic Radius"));
                    ui.add(egui::Slider::new(&mut game_config.settings.poison_area_radius, 1.0..=8.0)
                        .text("Poison Radius"));
                });

                ui.collapsing("Enemy Settings", |ui| {
                    ui.add(egui::Slider::new(&mut game_config.settings.enemy_movement_speed, 1.0..=8.0)
                        .text("Enemy Speed"));
                    ui.add(egui::Slider::new(&mut game_config.settings.enemy_max_health, 1.0..=20.0)
                        .text("Enemy Health"));
                    ui.add(egui::Slider::new(&mut game_config.settings.enemy_chase_distance, 3.0..=15.0)
                        .text("Chase Distance"));
                    ui.add(egui::Slider::new(&mut game_config.settings.score_per_enemy, 1..=50)
                        .text("Score per Kill"));
                });

                ui.collapsing("UI Settings", |ui| {
                    ui.add(egui::Slider::new(&mut game_config.settings.window_width, 800.0..=2560.0)
                        .text("Window Width"));
                    ui.add(egui::Slider::new(&mut game_config.settings.window_height, 600.0..=1440.0)
                        .text("Window Height"));
                    ui.add(egui::Slider::new(&mut game_config.settings.hud_font_size, 10.0..=24.0)
                        .text("HUD Font Size"));
                    ui.add(egui::Slider::new(&mut game_config.settings.tooltip_font_size, 8.0..=16.0)
                        .text("Tooltip Font Size"));
                    ui.add(egui::Slider::new(&mut game_config.settings.max_username_length, 5..=50)
                        .text("Max Username Length"));
                });

                ui.collapsing("Visual Settings", |ui| {
                    ui.add(egui::Slider::new(&mut game_config.settings.ambient_light_brightness, 100.0..=1000.0)
                        .text("Ambient Light"));
                    
                    ui.label("Health Bar Color:");
                    ui.color_edit_button_rgb(&mut game_config.settings.health_bar_color);
                    
                    ui.label("Mana Bar Color:");
                    ui.color_edit_button_rgb(&mut game_config.settings.mana_bar_color);
                    
                    ui.label("Energy Bar Color:");
                    ui.color_edit_button_rgb(&mut game_config.settings.energy_bar_color);
                });

                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Apply & Save").clicked() {
                        if let Err(err) = save_config(&game_config) {
                            eprintln!("Warning: Failed to save config: {}", err);
                        }
                    }
                    
                    if ui.button("Reset to Defaults").clicked() {
                        game_config.settings = GameSettings::default();
                    }
                    
                    if ui.button("Back to Menu").clicked() {
                        next_state.set(GameState::MainMenu);
                    }
                });
            });
        });
    }
}