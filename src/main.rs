use std::time::Duration;

use serde::{Deserialize, Serialize};

use bevy::{
    asset::ChangeWatcher,
    prelude::{shape::Quad, *},
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    window::{PrimaryWindow, WindowResized, WindowResolution},
};

fn main() {
    let settings = std::fs::read_to_string("window.json")
        .ok()
        .and_then(|s| serde_json::from_str::<WindowSettings>(&s).ok())
        .unwrap_or_default();
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: Some(ChangeWatcher {
                        delay: Duration::from_millis(100),
                    }),
                    ..Default::default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: settings.position,
                        resolution: settings.resolution,
                        ..default()
                    }),
                    ..default()
                }),
            Material2dPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, spawn)
        .add_systems(PreUpdate, watch_window)
        .run();
}

#[derive(Resource, Serialize, Deserialize, Default)]
struct WindowSettings {
    position: WindowPosition,
    resolution: WindowResolution,
}

fn spawn(
    mut commands: Commands,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle::default());

    let flat_size = Vec2::splat(800.0);
    let edge_size = Vec2::splat(70.0);
    let full_size = flat_size + edge_size;
    let mesh: Mesh = Quad::new(full_size).into();
    let mesh = Mesh2dHandle(meshes.add(mesh));

    let material = materials.add(CustomMaterial {
        color: Color::rgba(0.03, 0.03, 0.0, 0.6),
        flat_size,
        edge_size,
    });
    commands.spawn(MaterialMesh2dBundle {
        mesh,
        material,
        transform: Transform::from_translation(Vec3::new(-50.0, -50.0, -0.01)),
        ..default()
    });
}

fn watch_window(
    window: Query<&Window, With<PrimaryWindow>>,
    window_moved: EventReader<WindowMoved>,
    window_resized: EventReader<WindowResized>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    if window_moved.is_empty() && window_resized.is_empty() {
        return;
    }
    let settings = WindowSettings {
        position: window.position,
        resolution: (window.width(), window.height()).into(),
    };
    let settings = serde_json::to_string(&settings).ok();
    settings.and_then(|s| std::fs::write("window.json", s).ok());
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "51ebc8de-6c0f-481c-8e60-1d173e5cb80c"]
struct CustomMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(0)]
    flat_size: Vec2,
    #[uniform(0)]
    edge_size: Vec2,
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "custom_material.wgsl".into()
    }
}
