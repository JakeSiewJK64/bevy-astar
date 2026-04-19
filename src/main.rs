use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup, Startup, Update},
    asset::Assets,
    camera::{Camera2d, ClearColor},
    color::Color,
    ecs::{
        component::Component,
        event::Event,
        observer::On,
        system::{Commands, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::primitives::Rectangle,
    mesh::{Mesh, Mesh2d},
    sprite_render::{ColorMaterial, MeshMaterial2d},
    transform::components::Transform,
    utils::default,
    window::{Window, WindowPlugin},
};

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

const WIDTH: f32 = 50.;
const MARGIN: f32 = 1.;

#[derive(Component)]
struct Meta {
    x: i32,
    y: i32,
}

fn color_square(
    event: On<CustomEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&Meta, &MeshMaterial2d<ColorMaterial>)>,
) {
    let target_x = event.meta.x;
    let target_y = event.meta.y;
    bevy::log::info!("targeting x: {}, y: {}", target_x, target_y);

    for (item, material_handle) in query.iter_mut() {
        if item.x == target_x
            && item.y == target_y
            && let Some(material) = materials.get_mut(material_handle)
        {
            material.color = Color::WHITE;
            return;
        }
    }
}

#[derive(Event)]
struct CustomEvent {
    meta: Meta,
}

fn read_input(input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if input.just_pressed(KeyCode::Space) {
        commands.trigger(CustomEvent {
            meta: Meta { x: 10, y: 5 },
        });
    }
}

/// spawns square grids
fn spawn_squares(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    window: Query<&Window>,
) {
    // get window meta
    let window: &Window = window.single().unwrap();
    let width: f32 = window.resolution.width(); // 1080
    let height: f32 = window.resolution.height();

    // calculate top left corner
    let start_x: f32 = -width / 2.0 + WIDTH / 2.0;
    let start_y: f32 = height / 2.0 - WIDTH / 2.0;

    let x_total: i32 = 20;
    let y_total: i32 = 10;
    let color: Color = Color::srgb(1., 0., 0.);

    for i in 0..x_total {
        let x_pos: f32 = start_x + (i as f32 * (WIDTH + MARGIN));

        // iterate y
        for y in 0..y_total {
            let y_pos: f32 = start_y - (y as f32 * (WIDTH + MARGIN));
            let rect_mesh = meshes.add(Rectangle::new(WIDTH, WIDTH));

            commands.spawn((
                Mesh2d(rect_mesh),
                MeshMaterial2d(materials.add(color)),
                Transform::from_xyz(x_pos, y_pos, 0.),
                Meta { x: i, y },
            ));
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "starter-top-down-2d".to_string(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, (setup_camera, spawn_squares))
        .add_systems(Update, read_input)
        .add_observer(color_square)
        .run();
}
