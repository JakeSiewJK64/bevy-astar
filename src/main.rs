use bevy::{
    DefaultPlugins,
    app::{App, PluginGroup, Startup, Update},
    asset::Assets,
    camera::{Camera2d, ClearColor},
    color::Color,
    ecs::{
        system::{Commands, Query, ResMut},
    },
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
            ));
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_systems(Startup, setup_camera)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "starter-top-down-2d".to_string(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, spawn_squares)
        .run();
}
