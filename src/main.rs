use bevy::{
    DefaultPlugins,
    app::{App, AppExit, PluginGroup, PostStartup, Startup, Update},
    asset::Assets,
    camera::Camera2d,
    color::{
        Color,
        palettes::css::{BROWN, LIMEGREEN, WHITE},
    },
    ecs::{
        component::Component,
        event::Event,
        message::MessageWriter,
        observer::On,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::primitives::Rectangle,
    mesh::{Mesh, Mesh2d},
    sprite_render::{ColorMaterial, MeshMaterial2d},
    text::{TextColor, TextSpan},
    time::{Time, Timer},
    transform::components::Transform,
    ui::{PositionType, px, widget::Text},
    utils::default,
    window::{Window, WindowPlugin},
};
use rand::Rng;

pub mod astar;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub const ALLOW_DIAGONAL: bool = true;
pub const TOTAL_X: i32 = 50;
pub const TOTAL_Y: i32 = 20;

const GOAL: Coordinate = Coordinate {
    x: 30,
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
const WIDTH: f32 = 10.;
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

fn color_start_goal_nodes(commands: &mut Commands, start: Coordinate, goal: Coordinate) {
    commands.trigger(ColorSquareEvent {
        coordinate: start,
        color: Color::from(LIMEGREEN),
    });
    commands.trigger(ColorSquareEvent {
        coordinate: goal,
        color: Color::from(BROWN),
    });
}

fn clear_node_colors(commands: &mut Commands, nodes: &Vec<Coordinate>) {
    for node in nodes {
        commands.trigger(ColorSquareEvent {
            color: Color::WHITE,
            coordinate: Coordinate {
                x: node.x,
                y: node.y,
                ..Default::default()
            },
        })
    }
}

#[derive(Component)]
struct SpeedUpText;

fn read_input(
    input: Res<ButtonInput<KeyCode>>,
    mut exit: MessageWriter<AppExit>,
    mut global_state: ResMut<GlobalState>,
    mut commands: Commands,
    mut speed_up_text_query: Query<&mut TextSpan, With<SpeedUpText>>,
) {
    if input.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
    }

    // todo: speed up simulation
    if input.just_pressed(KeyCode::KeyW) {
        global_state.sped_up = !global_state.sped_up;

        global_state.timer = Timer::from_seconds(
            if global_state.sped_up {
                TIMER_INTERVAL / 4.
            } else {
                TIMER_INTERVAL
            },
            bevy::time::TimerMode::Repeating,
        );

        let mut speed_up_text = speed_up_text_query.single_mut().unwrap();
        speed_up_text.replace_range(
            ..,
            if global_state.sped_up {
                "<w> Slow down\n"
            } else {
                "<w> Speed up\n"
            },
        );
    }

    // todo: change goal coordinate and reset map
    if input.just_pressed(KeyCode::Space) {
        clear_node_colors(&mut commands, &global_state.frontier);
        clear_node_colors(&mut commands, &global_state.expanded);

        // todo: clear the previous goal node
        commands.trigger(ColorSquareEvent {
            color: Color::WHITE,
            coordinate: Coordinate {
                x: global_state.end.x,
                y: global_state.end.y,
                ..Default::default()
            },
        });
        // todo: reset global data
        let mut rng = rand::thread_rng();
        global_state.expanded.clear();
        global_state.frontier.clear();

        let end = Coordinate {
            x: rng.gen_range(0..TOTAL_X),
            y: rng.gen_range(0..TOTAL_Y),
            ..Default::default()
        };
        let start = Coordinate {
            x: rng.gen_range(0..TOTAL_X),
            y: rng.gen_range(0..TOTAL_Y),
            ..Default::default()
        };

        global_state.end = end;
        global_state.start = start;
        global_state.frontier.push(start);

        // todo: color targets
        color_start_goal_nodes(&mut commands, start, end);
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
    let start_x: f32 = -width / 4.0 + WIDTH / 4.0;
    let start_y: f32 = height / 4.0 - WIDTH / 4.0;

    for i in 0..TOTAL_X {
        let x_pos: f32 = start_x + (i as f32 * (WIDTH + MARGIN));

        // iterate y
        for y in 0..TOTAL_Y {
            let y_pos: f32 = start_y - (y as f32 * (WIDTH + MARGIN));
            let rect_mesh = meshes.add(Rectangle::new(WIDTH, WIDTH));

            commands.spawn((
                Mesh2d(rect_mesh),
                MeshMaterial2d(materials.add(Color::WHITE)),
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

fn diagnostics(mut commands: Commands) {
    // todo: render helper text
    commands
        .spawn((
            Text::new(""),
            bevy::prelude::Node {
                position_type: PositionType::Absolute,
                bottom: px(10.0),
                left: px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((TextSpan::new("<q> quit\n"), TextColor(WHITE.into())));
            parent.spawn((
                TextSpan::new("<w> Speed up\n"),
                SpeedUpText,
                TextColor(WHITE.into()),
            ));
            parent.spawn((
                TextSpan::new("<SPACE> randomize start and end nodes\n"),
                TextColor(WHITE.into()),
            ));
        });
    // todo: render diagnostics text
    commands
        .spawn((
            Text::new(""),
            bevy::prelude::Node {
                position_type: PositionType::Absolute,
                top: px(10.0),
                left: px(10.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextSpan::new(format!("Start: x={} y={}\n", START.x, START.y)),
                TextColor(WHITE.into()),
            ));
            parent.spawn((
                TextSpan::new(format!("Goal: x={} y={}", GOAL.x, GOAL.y)),
                TextColor(WHITE.into()),
            ));
        });
}

#[derive(Resource, Clone)]
struct GlobalState {
    start: Coordinate,
    end: Coordinate,
    timer: Timer,
    frontier: Vec<Coordinate>,
    expanded: Vec<Coordinate>,
    sped_up: bool,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            start: Default::default(),
            end: Default::default(),
            timer: Timer::from_seconds(TIMER_INTERVAL, bevy::time::TimerMode::Repeating),
            frontier: Vec::new(),
            expanded: Vec::new(),
            sped_up: false,
        }
    }
}

fn color_targets(mut commands: Commands, res: Res<GlobalState>) {
    color_start_goal_nodes(&mut commands, res.start, res.end);
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
        .add_systems(Startup, (setup_camera, spawn_squares, setup, diagnostics))
        .add_systems(Update, (read_input, update))
        .add_observer(color_square)
        .run();
}
