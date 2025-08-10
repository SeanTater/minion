pub mod combat;
pub mod egui_ui;
pub mod enemy;
pub mod environment;
pub mod map_loader;
pub mod player;
pub mod scene;
pub mod tooltips;
pub mod ui;
pub mod ui_common;

// Re-export all plugin types for main.rs
pub use combat::CombatPlugin;
pub use egui_ui::EguiUiPlugin;
pub use enemy::EnemyPlugin;
pub use environment::EnvironmentPlugin;
pub use map_loader::MapLoaderPlugin;
pub use player::PlayerPlugin;
pub use scene::ScenePlugin;
pub use tooltips::TooltipPlugin;
pub use ui::UiPlugin;
