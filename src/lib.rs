use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode, WindowResized, WindowResolution},
};
use serde::{Deserialize, Serialize};

pub struct WindowRestorePlugin;
const PATH: &str = "window.json";

/// Loads the window settings from `window.json`
pub fn set_window_plugin() -> WindowPlugin {
    let WindowSettings {
        mode,
        position,
        resolution,
    } = std::fs::read_to_string(PATH)
        .ok()
        .and_then(|s| serde_json::from_str::<WindowSettings>(&s).ok())
        .unwrap_or_default();

    WindowPlugin {
        primary_window: Some(Window {
            mode,
            position,
            resolution,
            ..default()
        }),
        ..default()
    }
}

/// Stores the window settings in `window.json`,
/// so that next run it can be restored.
///
/// ```no_run
/// App::new()
///    .add_plugins((
///        DefaultPlugins
///            .set(AssetPlugin {
///                 watch_for_changes: Some(ChangeWatcher {
///                     delay: Duration::from_millis(100),
///                 }),
///                 .Default::default()
///             })
///             .set(set_window_plugin()),
///         WindowRestorePlugin,
///     ))
///     .run();
/// ```
impl Plugin for WindowRestorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, watch_window);
    }
}

#[derive(Resource, Serialize, Deserialize, Default, Debug)]
struct WindowSettings {
    mode: WindowMode,
    position: WindowPosition,
    resolution: WindowResolution,
}

fn watch_window(
    window: Query<&Window, With<PrimaryWindow>>,
    mut last_window_mode: Local<WindowMode>,
    window_moved: EventReader<WindowMoved>,
    window_resized: EventReader<WindowResized>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    if *last_window_mode == window.mode && window_moved.is_empty() && window_resized.is_empty() {
        return;
    }
    *last_window_mode = window.mode;
    let settings = WindowSettings {
        mode: window.mode,
        position: window.position,
        resolution: (window.width(), window.height()).into(),
    };
    let settings = serde_json::to_string(&settings).ok();
    settings.and_then(|s| std::fs::write(PATH, s).ok());
}
