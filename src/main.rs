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

pub mod astar;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub const ALLOW_DIAGONAL: bool = true;
pub const TOTAL_X: i32 = 20;
pub const TOTAL_Y: i32 = 10;

const GOAL: Coordinate = Coordinate {
    x: 4,
    y: 4,
    cost: 0,
    score: 0,
};
const START: Coordinate = Coordinate {
    x: 0,
    y: 0,
    cost: 0,
    score: 0,
};
const TIMER_INTERVAL: f32 = 1.;
const WIDTH: f32 = 50.;
const MARGIN: f32 = 1.;

#[derive(Component, Clone, Copy, Default)]
pub struct Coordinate {
    x: i32,
    y: i32,
    cost: i32,
    score: i32,
}

fn color_square(
    event: On<ColorSquareEvent>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&Coordinate, &MeshMaterial2d<ColorMaterial>)>,
) {
    let target_x = event.coordinate.x;
    let target_y = event.coordinate.y;
    bevy::log::info!("Painting x: {}, y: {}", target_x, target_y);

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

    // for testing purposes, serves no functionality
    if input.just_pressed(KeyCode::Space) {
        commands.trigger(ColorSquareEvent {
            coordinate: Coordinate {
                x: 10,
                y: 5,
                ..default()
            },
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

    let x_total: i32 = TOTAL_X;
    let y_total: i32 = TOTAL_Y;
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
                Coordinate {
                    x: i,
                    y,
                    ..default()
                },
            ));
        }
    }
}

#[derive(Resource, Clone)]
struct GlobalState {
    start: Coordinate,
    end: Coordinate,
    timer: Timer,
    frontier: Vec<Coordinate>,
    expanded: Vec<Coordinate>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            start: Default::default(),
            end: Default::default(),
            timer: Timer::from_seconds(TIMER_INTERVAL, bevy::time::TimerMode::Repeating),
            frontier: Vec::new(),
            expanded: Vec::new(),
        }
    }
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

fn update(mut commands: Commands, time: Res<Time>, mut global_state: ResMut<GlobalState>) {
    // todo: only tick every 2 seconds.
    if !global_state.timer.tick(time.delta()).just_finished() {
        return;
    }

    let mut payload = astar::AStarPayload {
        goal: global_state.end,
        expanded: global_state.expanded.clone(),
        frontier: global_state.frontier.clone(),
    };

    match astar::astar_engine(&mut payload) {
        astar::AStarStatus::Found => {
            bevy::log::info!("target found");
        }
        astar::AStarStatus::Failed => {
            bevy::log::error!("No solutions found.");
        }
        astar::AStarStatus::Pending => {
            global_state.expanded = payload.expanded;
            global_state.frontier = payload.frontier;

            // todo: color frontier nodes
            for node in &global_state.frontier {
                commands.trigger(ColorSquareEvent {
                    coordinate: Coordinate {
                        x: node.x,
                        y: node.y,
                        ..Default::default()
                    },
                    color: Color::from(LIMEGREEN),
                });
            }
        }
    }
}

fn setup(mut global_state: ResMut<GlobalState>) {
    let last_node = global_state.start;

    // todo: add starting node as frontier
    global_state.frontier.push(last_node);
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
            end: GOAL,
            start: START,
            ..default()
        })
        .add_systems(PostStartup, color_targets)
        .add_systems(Startup, (setup_camera, spawn_squares, setup))
        .add_systems(Update, (read_input, update))
        .add_observer(color_square)
        .run();
}
