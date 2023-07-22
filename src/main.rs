use std::time::Duration;

use serde::{Deserialize, Serialize};

use bevy::{
    asset::ChangeWatcher,
    input::mouse::MouseWheel,
    prelude::{shape::Quad, *},
    reflect::{TypePath, TypeUuid},
    render::{
        camera::RenderTarget,
        render_resource::{AsBindGroup, ShaderRef},
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    window::{PrimaryWindow, WindowRef, WindowResized, WindowResolution},
};

fn main() {
    let settings = std::fs::read_to_string("window.json").ok();
    let settings = settings.and_then(|s| serde_json::from_str::<WindowSettings>(&s).ok());
    let settings = settings.unwrap_or_default();
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
        .init_resource::<WorldCursor>()
        .add_systems(Startup, spawn)
        .add_systems(PreUpdate, update_cursor)
        .add_systems(Update, watch_window)
        .add_systems(Update, pick)
        .add_systems(Update, resize)
        .add_systems(PostUpdate, synchronise_sizes)
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
    let entity = commands
        .spawn(MaterialMesh2dBundle {
            mesh,
            material,
            transform: Transform::from_translation(Vec3::new(-50.0, -50.0, -0.01)),
            ..default()
        })
        .id();

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(flat_size),
                ..default()
            },
            ..default()
        })
        .add_child(entity);
}

fn synchronise_sizes(
    objects: Query<&Sprite, Changed<Sprite>>,
    shadows: Query<(&Parent, &Handle<CustomMaterial>, &Mesh2dHandle)>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (parent, mat, mesh) in shadows.iter() {
        let Ok(sprite) = objects.get(parent.get()) else {
            continue;
        };
        let Some(material) = materials.get_mut(mat) else {
            continue;
        };
        let Some(mesh) = meshes.get_mut(&mesh.0) else {
            continue;
        };

        let flat_size = sprite.custom_size.unwrap();
        let edge_size = material.edge_size;
        let full_size = flat_size + edge_size;
        *mesh = Quad::new(full_size).into();
        material.flat_size = flat_size;
    }
}

#[derive(Resource, Default)]
struct WorldCursor(Vec2);

fn update_cursor(
    mut cursor: ResMut<WorldCursor>,
    // need to get window dimensions
    windows: Query<&Window>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    for (camera, camera_transform) in camera_q.iter() {
        // get the window that the camera is displaying to (or the primary window)
        let window = if let RenderTarget::Window(WindowRef::Entity(id)) = camera.target {
            windows.get(id).unwrap()
        } else {
            primary_window.single()
        };

        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            cursor.0 = world_position;
        }
    }
}

fn pick(
    mouse: Res<Input<MouseButton>>,
    // grabs the mouse cursor
    cursor: Res<WorldCursor>,
    mut query: Query<(Entity, &mut Transform, &Sprite)>,
    mut held: Local<Option<(Entity, Vec2)>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        for (entity, transform, sprite) in query.iter() {
            let quadrant = sprite.custom_size.unwrap() / 2.0;
            let origin = transform.translation.truncate();
            let top_left = origin - quadrant;
            let bottom_right = origin + quadrant;
            let rect = Rect {
                min: top_left,
                max: bottom_right,
            };
            if rect.contains(cursor.0) {
                *held = Some((entity, cursor.0 - origin));
                break;
            }
        }
    } else if mouse.pressed(MouseButton::Left) {
        if let Some((target, offset)) = *held {
            if let Ok((_, mut transform, _)) = query.get_mut(target) {
                transform.translation = (cursor.0 - offset).extend(transform.translation.z);
            };
        }
    } else if !mouse.pressed(MouseButton::Left) && held.is_some() {
        *held = None;
    }
}

fn resize(
    mut wheel: EventReader<MouseWheel>,
    input: Res<Input<KeyCode>>,
    // grabs the mouse cursor
    cursor: Res<WorldCursor>,
    mut query: Query<(&Transform, &mut Sprite)>,
) {
    if wheel.is_empty() {
        return;
    }

    let mut query_iter = query.iter_mut();
    let mut sprite = loop {
        let Some((transform, sprite)) = query_iter.next() else {
            break None;
        };
        let quadrant: Vec2 = sprite.custom_size.unwrap() / 2.0;
        let origin = transform.translation.truncate();
        let top_left = origin - quadrant;
        let bottom_right = origin + quadrant;
        let rect = Rect {
            min: top_left,
            max: bottom_right,
        };
        if rect.contains(cursor.0) {
            break Some(sprite);
        }
    };
    let Some(size) = sprite.as_deref_mut().and_then(|sprite| sprite.custom_size.as_mut()) else {
        return;
    };
    for scroll in wheel.iter() {
        if input.pressed(KeyCode::ShiftLeft) {
            size.y *= 1.0 + 0.1 * scroll.y;
        }

        if input.pressed(KeyCode::AltLeft) {
            size.x *= 1.0 + 0.1 * scroll.y;
        }
    }
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
