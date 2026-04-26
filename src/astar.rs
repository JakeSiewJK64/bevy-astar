use bevy::utils::default;

use crate::Coordinate;

pub struct AStarPayload {
    pub goal: Coordinate,
    pub frontier: Vec<Coordinate>,
    pub expanded: Vec<Coordinate>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum AStarStatus {
    Found,
    Failed,
    Pending,
}

fn get_manhattan_distance(coordinate: &Coordinate, goal: Coordinate) -> i32 {
    i32::abs(goal.x - coordinate.x) + i32::abs(goal.y - coordinate.y)
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
    if coordinate.y + 1 < crate::TOTAL_Y {
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
    if coordinate.x + 1 < crate::TOTAL_X {
        expanded.push(Coordinate {
            x: coordinate.x + 1,
            y: coordinate.y,
            ..default()
        })
    }

    expanded
}

pub fn astar_engine(payload: &mut AStarPayload) -> AStarStatus {
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

#[cfg(test)]
mod test {
    use crate::{
        Coordinate,
        astar::{AStarPayload, AStarStatus, astar_engine, get_manhattan_distance, get_neighbors},
    };

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
