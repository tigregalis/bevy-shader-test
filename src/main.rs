use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    prelude::{shape::Quad, *},
    reflect::{TypePath, TypeUuid},
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_shader_test::{set_window_plugin, WindowRestorePlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: Some(ChangeWatcher {
                        delay: Duration::from_millis(100),
                    }),
                    ..Default::default()
                })
                .set(set_window_plugin()),
            Material2dPlugin::<CustomMaterial>::default(),
            WindowRestorePlugin,
        ))
        .add_systems(Startup, spawn)
        .run();
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
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..default()
    });
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
