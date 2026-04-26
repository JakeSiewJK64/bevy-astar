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
const TIMER_INTERVAL: f32 = 0.25;
const TOTAL_X: i32 = 20;
const TOTAL_Y: i32 = 10;
const WIDTH: f32 = 50.;
const MARGIN: f32 = 1.;

#[derive(Component, Clone, Copy, Default)]
struct Coordinate {
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

fn get_neighbors(coordinate: &Coordinate) -> Vec<Coordinate> {
    let mut expanded: Vec<Coordinate> = Vec::new();

    // search up
    if coordinate.y - 1 > 0 {
        expanded.push(Coordinate {
            x: coordinate.x,
            y: coordinate.y - 1,
            ..default()
        })
    }

    // search down
    if coordinate.y + 1 < TOTAL_Y {
        expanded.push(Coordinate {
            x: coordinate.x,
            y: coordinate.y + 1,
            ..default()
        })
    }

    // search left
    if coordinate.x - 1 > 0 {
        expanded.push(Coordinate {
            x: coordinate.x - 1,
            y: coordinate.y,
            ..default()
        })
    }

    // search right
    if coordinate.x + 1 < TOTAL_X {
        expanded.push(Coordinate {
            x: coordinate.x + 1,
            y: coordinate.y,
            ..default()
        })
    }

    expanded
}

fn get_manhattan_distance(coordinate: &Coordinate, goal: Coordinate) -> i32 {
    i32::abs(goal.x - coordinate.x) + i32::abs(goal.y - coordinate.y)
}

fn update(mut commands: Commands, time: Res<Time>, mut global_state: ResMut<GlobalState>) {
    // todo: only tick every 2 seconds.
    if !global_state.timer.tick(time.delta()).just_finished() {
        return;
    }

    let mut payload = AStarPayload {
        goal: global_state.end,
        expanded: global_state.expanded.clone(),
        frontier: global_state.frontier.clone(),
    };

    match astar_engine(&mut payload) {
        AStarStatus::Found => {
            bevy::log::info!("target found");
        }
        AStarStatus::Failed => {
            bevy::log::error!("No solutions found.");
        }
        AStarStatus::Pending => {
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

struct AStarPayload {
    goal: Coordinate,
    frontier: Vec<Coordinate>,
    expanded: Vec<Coordinate>,
}

#[derive(PartialEq, Eq, Debug)]
enum AStarStatus {
    Found,
    Failed,
    Pending,
}

fn astar_engine(payload: &mut AStarPayload) -> AStarStatus {
    if payload.frontier.is_empty() {
        return AStarStatus::Failed;
    }

    // todo: sort frontier by cost
    payload.frontier.sort_by_key(|node| -node.score);

    // todo: remove first item from frontier and put into expanded
    if let Some(frontier_node) = payload.frontier.pop() {
        // todo: something else
        payload.expanded.push(frontier_node);

        // todo: get last item in expanded
        if let Some(expanded_node) = payload.expanded.last() {
            // todo: if goal found, stop algorithm
            if expanded_node.x == payload.goal.x && expanded_node.y == payload.goal.y {
                return AStarStatus::Found;
            }

            let cost = expanded_node.cost + 1;

            // todo: iterate neighbors
            for neighbor in get_neighbors(expanded_node) {
                let mut duplicate_found = false;

                // todo: if neighbor not in expanded list, add them
                for node in payload.expanded.iter() {
                    if node.x == neighbor.x && node.y == neighbor.y {
                        duplicate_found = true;
                        break;
                    }
                }

                if !duplicate_found {
                    let h_cost = get_manhattan_distance(&neighbor, payload.goal);
                    payload.expanded.push(neighbor);
                    payload.frontier.push(Coordinate {
                        x: neighbor.x,
                        y: neighbor.y,
                        cost,
                        score: cost + h_cost,
                    });
                }
            }

            return AStarStatus::Pending;
        }
    }

    AStarStatus::Failed
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

#[cfg(test)]
mod test {
    use crate::{
        AStarPayload, AStarStatus, Coordinate, astar_engine, get_manhattan_distance, get_neighbors,
    };

    fn print_coordinate_list(coordinates: Vec<Coordinate>) {
        for coordinate in coordinates {
            println!("x: {}, y: {}", coordinate.x, coordinate.y);
        }
    }

    #[test]
    fn test_astar_engine() {
        let goal = Coordinate {
            x: 4,
            y: 4,
            ..Default::default()
        };
        let start = Coordinate {
            x: 4,
            y: 4,
            ..Default::default()
        };
        let mut payload = AStarPayload {
            expanded: vec![],
            frontier: vec![start],
            goal,
        };

        let res = astar_engine(&mut payload);
        assert_eq!(res, AStarStatus::Found);

        // todo: 2nd iteration
        let goal = Coordinate {
            x: 4,
            y: 4,
            ..Default::default()
        };
        let start = Coordinate {
            x: 0,
            y: 0,
            ..Default::default()
        };
        let mut payload = AStarPayload {
            expanded: vec![],
            frontier: vec![start],
            goal,
        };

        // the algorithm should ideally take at most 8 iterations to reach target
        for epoch in 0..8 {
            println!("expanded list for epoch: {}", epoch);
            print_coordinate_list(payload.expanded.clone());
            astar_engine(&mut payload);
            if epoch == 7 {
                let res = astar_engine(&mut payload);
                assert_eq!(res, AStarStatus::Found);
            }
        }
    }

    #[test]
    fn test_get_manhattan_distance() {
        // node: no diagonal movement currently
        let coordinate = Coordinate {
            x: 0,
            y: 0,
            ..Default::default()
        };
        let goal = Coordinate {
            x: 2,
            y: 2,
            ..Default::default()
        };
        let distance = get_manhattan_distance(&coordinate, goal);

        assert_eq!(distance, 4);
    }

    #[test]
    fn test_get_neighbors() {
        // todo: initialize 0,0 coordinate
        let coordinate = Coordinate {
            x: 5,
            y: 5,
            ..Default::default()
        };

        let neighbors = get_neighbors(&coordinate);

        for (index, neighbor) in neighbors.iter().enumerate() {
            // test: search up
            if index == 0 {
                assert_eq!(neighbor.x, 5);
                assert_eq!(neighbor.y, 4);
            }

            // test: search down
            if index == 1 {
                assert_eq!(neighbor.x, 5);
                assert_eq!(neighbor.y, 6);
            }

            // test: search left
            if index == 2 {
                assert_eq!(neighbor.x, 4);
                assert_eq!(neighbor.y, 5);
            }

            // test: search right
            if index == 3 {
                assert_eq!(neighbor.x, 6);
                assert_eq!(neighbor.y, 5);
            }
        }
    }
}
