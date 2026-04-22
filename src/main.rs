use bevy::{
    DefaultPlugins,
    app::{App, AppExit, PluginGroup, PostStartup, Startup, Update},
    asset::Assets,
    camera::{Camera2d, ClearColor},
    color::{
        Color,
        palettes::css::{BROWN, LIMEGREEN},
    },
    ecs::{
        component::Component,
        event::Event,
        message::MessageWriter,
        observer::On,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::primitives::Rectangle,
    mesh::{Mesh, Mesh2d},
    sprite_render::{ColorMaterial, MeshMaterial2d},
    time::{Time, Timer},
    transform::components::Transform,
    utils::default,
    window::{Window, WindowPlugin},
};

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

const WIDTH: f32 = 50.;
const MARGIN: f32 = 1.;

#[derive(Component, Clone, Copy)]
struct Coordinate {
    x: i32,
    y: i32,
}

fn color_square(
    event: On<ColorSquareEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&Coordinate, &MeshMaterial2d<ColorMaterial>)>,
) {
    let target_x = event.coordinate.x;
    let target_y = event.coordinate.y;
    bevy::log::info!("targeting x: {}, y: {}", target_x, target_y);

    for (item, material_handle) in query.iter_mut() {
        if item.x == target_x
            && item.y == target_y
            && let Some(material) = materials.get_mut(material_handle)
        {
            material.color = event.color;
            return;
        }
    }
}

#[derive(Event)]
struct ColorSquareEvent {
    coordinate: Coordinate,
    color: Color,
}

fn read_input(
    input: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }

    if input.just_pressed(KeyCode::Space) {
        commands.trigger(ColorSquareEvent {
            coordinate: Coordinate { x: 10, y: 5 },
            color: Color::WHITE,
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
    // get window coordinate
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
                Coordinate { x: i, y },
            ));
        }
    }
}

#[derive(Resource, Clone)]
struct GlobalState {
    start: Coordinate,
    end: Coordinate,
    timer: Timer,
    last_node: Coordinate,
}

fn color_targets(mut commands: Commands, res: Res<GlobalState>) {
    commands.trigger(ColorSquareEvent {
        coordinate: res.start,
        color: Color::from(LIMEGREEN),
    });
    commands.trigger(ColorSquareEvent {
        coordinate: res.end,
        color: Color::from(BROWN),
    });
}

fn astar_engine(mut commands: Commands, time: Res<Time>, mut global_state: ResMut<GlobalState>) {
    if !global_state.timer.tick(time.delta()).just_finished() {
        return;
    }

    // todo: if more than 2, move back
    if global_state.last_node.x > 2 {
        // todo: decrement node x by 1
        global_state.last_node.x -= 1;

        // todo: do something when timer finish
        // todo: print path movement
        commands.trigger(ColorSquareEvent {
            color: Color::WHITE,
            coordinate: global_state.last_node,
        });
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
        .insert_resource(GlobalState {
            end: Coordinate { x: 10, y: 0 },
            start: Coordinate { x: 15, y: 0 },
            last_node: Coordinate { x: 15, y: 0 },
            timer: Timer::from_seconds(1., bevy::time::TimerMode::Repeating),
        })
        .add_systems(PostStartup, color_targets)
        .add_systems(Startup, (setup_camera, spawn_squares))
        .add_systems(Update, (read_input, astar_engine))
        .add_observer(color_square)
        .run();
}
