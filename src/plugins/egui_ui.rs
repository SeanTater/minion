use crate::config::range_types::*;
use crate::config::save_config;
use crate::plugins::ui_common::handle_exit_events;
use crate::resources::{GameConfig, GameSettings, GameState};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

/// Macro to create sliders for range-safe types
macro_rules! range_safe_slider {
    ($ui:expr, $value:expr, $range:expr, $text:expr, $suffix:expr, $type:ty) => {{
        let mut temp_value = $value.get();
        let response = $ui.add(
            egui::Slider::new(&mut temp_value, $range)
                .text($text)
                .suffix($suffix),
        );
        *$value = <$type>::new(temp_value);
        response
    }};
}

pub struct EguiUiPlugin;

impl Plugin for EguiUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .add_systems(Startup, setup_ui_camera)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    main_menu_egui_system.run_if(in_state(GameState::MainMenu)),
                    settings_egui_system.run_if(in_state(GameState::Settings)),
                    handle_exit_events,
                ),
            );
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

fn main_menu_egui_system(
    mut contexts: EguiContexts,
    game_config: Res<GameConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        // Set a dark theme with larger default fonts
        ctx.set_visuals(egui::Visuals::dark());

        // Set larger default font size
        ctx.style_mut(|style| {
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(24.0, egui::FontFamily::Proportional),
            );
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(20, 20, 30)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);

                    // Large dramatic title
                    ui.add(egui::Label::new(
                        egui::RichText::new("MINION")
                            .size(84.0)
                            .color(egui::Color32::from_rgb(220, 50, 50))
                            .strong(),
                    ));

                    ui.add_space(20.0);
                    ui.add(egui::Label::new(
                        egui::RichText::new("Action RPG")
                            .size(28.0)
                            .color(egui::Color32::from_rgb(180, 180, 180))
                            .italics(),
                    ));

                    ui.add_space(20.0);

                    // Show current username if set
                    if !game_config.username.is_empty() {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!(
                                "Welcome back, {username}!",
                                username = game_config.username
                            ))
                            .size(20.0)
                            .color(egui::Color32::from_rgb(100, 200, 100)),
                        ));
                        ui.add_space(40.0);
                    } else {
                        ui.add_space(60.0);
                    }

                    // Centered buttons with better styling
                    ui.vertical_centered(|ui| {
                        let button_size = [160.0, 50.0];
                        ui.spacing_mut().item_spacing.y = 15.0;

                        if ui
                            .add_sized(
                                button_size,
                                egui::Button::new(
                                    egui::RichText::new("⚔ PLAY").size(20.0).strong(),
                                )
                                .fill(egui::Color32::from_rgb(60, 120, 60)),
                            )
                            .clicked()
                        {
                            next_state.set(GameState::Playing);
                        }

                        if ui
                            .add_sized(
                                button_size,
                                egui::Button::new(egui::RichText::new("⚙ SETTINGS").size(18.0))
                                    .fill(egui::Color32::from_rgb(80, 80, 120)),
                            )
                            .clicked()
                        {
                            next_state.set(GameState::Settings);
                        }

                        if ui
                            .add_sized(
                                button_size,
                                egui::Button::new(egui::RichText::new("🚪 QUIT").size(18.0))
                                    .fill(egui::Color32::from_rgb(120, 60, 60)),
                            )
                            .clicked()
                        {
                            exit.write(AppExit::Success);
                        }
                    });

                    ui.add_space(60.0);
                    ui.add(egui::Label::new(
                        egui::RichText::new("Press Escape to exit")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(120, 120, 120))
                            .italics(),
                    ));
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
        // Set larger default fonts for settings too
        ctx.set_visuals(egui::Visuals::dark());
        ctx.style_mut(|style| {
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(16.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(24.0, egui::FontFamily::Proportional),
            );
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(egui::Color32::from_rgb(25, 25, 35)))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    // Settings title
                    ui.add(egui::Label::new(
                        egui::RichText::new("⚙ GAME SETTINGS")
                            .size(52.0)
                            .color(egui::Color32::from_rgb(200, 180, 100))
                            .strong(),
                    ));

                    ui.add_space(30.0);
                });

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Use a grid layout for better organization with full width expansion
                        ui.columns(2, |columns| {
                            // Left column - force full width
                            columns[0].allocate_ui_with_layout(
                                [columns[0].available_width(), columns[0].available_height()]
                                    .into(),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    // Player Settings - default open with username
                                    egui::CollapsingHeader::new(
                                        egui::RichText::new("🏃 Player Settings")
                                            .size(22.0)
                                            .color(egui::Color32::from_rgb(100, 200, 100)),
                                    )
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.label(egui::RichText::new("Username:").size(18.0));
                                        ui.add_sized(
                                            [ui.available_width() - 10.0, 32.0],
                                            egui::TextEdit::singleline(&mut game_config.username)
                                                .hint_text("Enter your hero name..."),
                                        );
                                        ui.add_space(10.0);

                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.player_movement_speed,
                                            1.0..=10.0,
                                            "Movement Speed",
                                            " units/s",
                                            MovementSpeed
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.player_max_health,
                                            10.0..=200.0,
                                            "Max Health",
                                            " HP",
                                            HealthValue
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.player_max_mana,
                                            10.0..=100.0,
                                            "Max Mana",
                                            " MP",
                                            ManaValue
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.player_max_energy,
                                            10.0..=200.0,
                                            "Max Energy",
                                            " EN",
                                            EnergyValue
                                        );
                                    });

                                    ui.add_space(20.0);

                                    // Combat Settings - default open
                                    egui::CollapsingHeader::new(
                                        egui::RichText::new("⚔ Combat Settings")
                                            .size(22.0)
                                            .color(egui::Color32::from_rgb(200, 100, 100)),
                                    )
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.bullet_speed,
                                            5.0..=30.0,
                                            "Bullet Speed",
                                            " units/s",
                                            BulletSpeed
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.bullet_damage,
                                            0.5..=10.0,
                                            "Bullet Damage",
                                            " DMG",
                                            BulletDamage
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.bullet_lifetime,
                                            1.0..=8.0,
                                            "Bullet Lifetime",
                                            " s",
                                            Lifetime
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.magic_damage_per_second,
                                            50.0..=500.0,
                                            "Magic DPS",
                                            " DMG/s",
                                            DamagePerSecond
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.poison_damage_per_second,
                                            20.0..=300.0,
                                            "Poison DPS",
                                            " DMG/s",
                                            DamagePerSecond
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.magic_area_radius,
                                            1.0..=8.0,
                                            "Magic Radius",
                                            " units",
                                            AreaRadius
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.poison_area_radius,
                                            1.0..=10.0,
                                            "Poison Radius",
                                            " units",
                                            AreaRadius
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.magic_area_duration,
                                            0.5..=10.0,
                                            "Magic Duration",
                                            " s",
                                            AreaDuration
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.poison_area_duration,
                                            0.5..=15.0,
                                            "Poison Duration",
                                            " s",
                                            AreaDuration
                                        );
                                    });
                                },
                            );

                            // Right column - force full width
                            columns[1].allocate_ui_with_layout(
                                [columns[1].available_width(), columns[1].available_height()]
                                    .into(),
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    // Enemy Settings - default open
                                    egui::CollapsingHeader::new(
                                        egui::RichText::new("👹 Enemy Settings")
                                            .size(22.0)
                                            .color(egui::Color32::from_rgb(200, 150, 100)),
                                    )
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.enemy_movement_speed,
                                            1.0..=8.0,
                                            "Movement Speed",
                                            " units/s",
                                            MovementSpeed
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.enemy_max_health,
                                            1.0..=20.0,
                                            "Max Health",
                                            " HP",
                                            HealthValue
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.enemy_max_mana,
                                            5.0..=50.0,
                                            "Max Mana",
                                            " MP",
                                            ManaValue
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.enemy_max_energy,
                                            10.0..=100.0,
                                            "Max Energy",
                                            " EN",
                                            EnergyValue
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.enemy_chase_distance,
                                            3.0..=15.0,
                                            "Chase Distance",
                                            " units",
                                            ChaseDistance
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.enemy_collision_distance,
                                            0.5..=3.0,
                                            "Collision Distance",
                                            " units",
                                            CollisionDistance
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.bullet_collision_distance,
                                            0.2..=2.0,
                                            "Bullet Collision Distance",
                                            " units",
                                            CollisionDistance
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.enemy_spawn_distance_min,
                                            2.0..=10.0,
                                            "Min Spawn Distance",
                                            " units",
                                            SpawnDistanceMin
                                        );
                                        range_safe_slider!(
                                            ui,
                                            &mut game_config.settings.enemy_spawn_distance_max,
                                            8.0..=25.0,
                                            "Max Spawn Distance",
                                            " units",
                                            SpawnDistanceMax
                                        );
                                        ui.add(
                                            egui::Slider::new(
                                                &mut game_config.settings.score_per_enemy,
                                                1..=50,
                                            )
                                            .text("Score per Kill")
                                            .suffix(" pts"),
                                        );
                                    });

                                    ui.add_space(20.0);

                                    // UI Settings - default closed
                                    egui::CollapsingHeader::new(
                                        egui::RichText::new("🖥 UI Settings")
                                            .size(22.0)
                                            .color(egui::Color32::from_rgb(100, 150, 200)),
                                    )
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::Slider::new(
                                                &mut game_config.settings.window_width,
                                                800.0..=2560.0,
                                            )
                                            .text("Window Width")
                                            .suffix(" px"),
                                        );
                                        ui.add(
                                            egui::Slider::new(
                                                &mut game_config.settings.window_height,
                                                600.0..=1440.0,
                                            )
                                            .text("Window Height")
                                            .suffix(" px"),
                                        );
                                        ui.add(
                                            egui::Slider::new(
                                                &mut game_config.settings.hud_font_size,
                                                10.0..=24.0,
                                            )
                                            .text("HUD Font Size")
                                            .suffix(" px"),
                                        );
                                        ui.add(
                                            egui::Slider::new(
                                                &mut game_config.settings.tooltip_font_size,
                                                8.0..=16.0,
                                            )
                                            .text("Tooltip Font Size")
                                            .suffix(" px"),
                                        );
                                        ui.add(
                                            egui::Slider::new(
                                                &mut game_config.settings.max_username_length,
                                                5..=50,
                                            )
                                            .text("Max Username Length")
                                            .suffix(" chars"),
                                        );
                                    });

                                    ui.add_space(20.0);

                                    // Visual Settings - default closed
                                    egui::CollapsingHeader::new(
                                        egui::RichText::new("🎨 Visual Settings")
                                            .size(22.0)
                                            .color(egui::Color32::from_rgb(200, 100, 200)),
                                    )
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::Slider::new(
                                                &mut game_config.settings.ambient_light_brightness,
                                                100.0..=1000.0,
                                            )
                                            .text("Ambient Light")
                                            .suffix(" lux"),
                                        );

                                        ui.separator();
                                        ui.label(
                                            egui::RichText::new("Bar Colors:").size(18.0).strong(),
                                        );

                                        ui.horizontal(|ui| {
                                            ui.label("Health:");
                                            ui.color_edit_button_rgb(
                                                &mut game_config.settings.health_bar_color,
                                            );
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Mana:");
                                            ui.color_edit_button_rgb(
                                                &mut game_config.settings.mana_bar_color,
                                            );
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Energy:");
                                            ui.color_edit_button_rgb(
                                                &mut game_config.settings.energy_bar_color,
                                            );
                                        });
                                    });
                                },
                            );
                        });

                        ui.add_space(30.0);
                        ui.separator();
                        ui.add_space(20.0);

                        // Bottom buttons - centered
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                let button_size = [160.0, 50.0];
                                ui.spacing_mut().item_spacing.x = 15.0;

                                if ui
                                    .add_sized(
                                        button_size,
                                        egui::Button::new(
                                            egui::RichText::new("💾 APPLY & SAVE").size(16.0),
                                        )
                                        .fill(egui::Color32::from_rgb(60, 120, 60)),
                                    )
                                    .clicked()
                                {
                                    if let Err(err) = save_config(&game_config) {
                                        eprintln!("Warning: Failed to save config: {err}");
                                    }
                                }

                                if ui
                                    .add_sized(
                                        button_size,
                                        egui::Button::new(
                                            egui::RichText::new("🔄 RESET").size(16.0),
                                        )
                                        .fill(egui::Color32::from_rgb(120, 80, 60)),
                                    )
                                    .clicked()
                                {
                                    game_config.settings = GameSettings::default();
                                }

                                if ui
                                    .add_sized(
                                        button_size,
                                        egui::Button::new(egui::RichText::new("⬅ BACK").size(16.0))
                                            .fill(egui::Color32::from_rgb(80, 80, 120)),
                                    )
                                    .clicked()
                                {
                                    next_state.set(GameState::MainMenu);
                                }
                            });
                        });

                        ui.add_space(20.0);
                    });
            });
    }
}
