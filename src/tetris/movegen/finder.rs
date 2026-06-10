use std::cmp::Ordering;
use std::collections::BinaryHeap;

use ahash::AHashMap;

use crate::tetris::model::rules::{GameRules, SonicDrop};
use crate::tetris::model::*;
use crate::tetris::movegen::collision_maps::CollisionMaps;
use crate::tetris::movegen::kicks;

pub fn find_moves(board: &Board, piece: Piece, rules: &GameRules) -> Vec<(Placement, u32)> {
    puffin::profile_function!();
    let mut queue = BinaryHeap::new();
    let mut values = AHashMap::new();
    let mut underground_locks = AHashMap::new();
    let mut locks = Vec::with_capacity(64);
    let collision_map = CollisionMaps::new(board, piece);

    let fast_mode = rules.sonic_drop == SonicDrop::Only;
    if fast_mode {
        for &rotation in &[
            Rotation::North,
            Rotation::East,
            Rotation::South,
            Rotation::West,
        ] {
            for x in 0..10 {
                let mut location = PieceLocation {
                    piece,
                    rotation,
                    x,
                    y: rules.spawn_y,
                };
                if collision_map.obstructed(location) {
                    continue;
                }
                let distance = location.drop_distance(board);
                location.y -= distance;
                let mv = Placement {
                    location,
                    spin: Spin::None,
                };

                let mut update_position =
                    update_position(&mut queue, &mut values, fast_mode, board);

                if let Some(mv) = shift(location, &collision_map, -1) {
                    update_position(mv, distance as u32);
                }
                if let Some(mv) = shift(location, &collision_map, 1) {
                    update_position(mv, distance as u32);
                }
                for rotation in [location.rotation.cw(), location.rotation.ccw()] {
                    if let Some(mv) =
                        rotate_to(location, rotation, &collision_map, board, rules.kickset)
                    {
                        update_position(mv, distance as u32);
                    }
                }
                if rules.rot180 {
                    if let Some(mv) = rotate_to(
                        location,
                        location.rotation.flip(),
                        &collision_map,
                        board,
                        rules.kickset,
                    ) {
                        update_position(mv, distance as u32);
                    }
                }

                if location.canonical_form() == location {
                    locks.push((mv, 0));
                }
            }
        }
    } else {
        let mut spawned = PieceLocation {
            piece,
            rotation: Rotation::North,
            x: rules.spawn_x,
            y: rules.spawn_y,
        };
        if collision_map.obstructed(spawned) {
            spawned.y += 1;
            if collision_map.obstructed(spawned) {
                return vec![];
            }
        }
        let spawned = Placement {
            location: spawned,
            spin: Spin::None,
        };
        queue.push(Intermediate {
            soft_drops: 0,
            mv: spawned,
        });
        values.insert(spawned, 0);
    }

    while let Some(expand) = queue.pop() {
        if expand.soft_drops != values.get(&expand.mv).copied().unwrap_or(40) {
            continue;
        }

        let drop_dist = expand.mv.location.drop_distance(board);
        let dropped = Placement {
            location: PieceLocation {
                y: expand.mv.location.y - drop_dist,
                ..expand.mv.location
            },
            spin: if drop_dist == 0 {
                expand.mv.spin
            } else {
                Spin::None
            },
        };

        let key = Placement {
            location: dropped.location.canonical_form(),
            ..dropped
        };
        let entry = underground_locks
            .entry(key)
            .or_insert((dropped, expand.soft_drops));
        if expand.soft_drops < entry.1 {
            *entry = (dropped, expand.soft_drops);
        }

        let mut update_position = update_position(&mut queue, &mut values, fast_mode, board);

        update_position(dropped, expand.soft_drops + drop_dist as u32);

        if let Some(mv) = shift(expand.mv.location, &collision_map, -1) {
            update_position(mv, expand.soft_drops);
        }
        if let Some(mv) = shift(expand.mv.location, &collision_map, 1) {
            update_position(mv, expand.soft_drops);
        }
        for rotation in [
            expand.mv.location.rotation.cw(),
            expand.mv.location.rotation.ccw(),
        ] {
            if let Some(mv) = rotate_to(
                expand.mv.location,
                rotation,
                &collision_map,
                board,
                rules.kickset,
            ) {
                update_position(mv, expand.soft_drops);
            }
        }
        if rules.rot180 {
            if let Some(mv) = rotate_to(
                expand.mv.location,
                expand.mv.location.rotation.flip(),
                &collision_map,
                board,
                rules.kickset,
            ) {
                update_position(mv, expand.soft_drops);
            }
        }
    }

    locks.extend(underground_locks.drain().map(|(_, value)| value));
    locks
}

fn update_position<'a>(
    queue: &'a mut BinaryHeap<Intermediate>,
    values: &'a mut AHashMap<Placement, u32>,
    fast_mode: bool,
    board: &'a Board,
) -> impl FnMut(Placement, u32) + 'a {
    move |target: Placement, soft_drops: u32| {
        if fast_mode && target.location.above_stack(board) {
            return;
        }
        let prev_sds = values.entry(target).or_insert(40);
        if soft_drops < *prev_sds {
            *prev_sds = soft_drops;
            queue.push(Intermediate {
                soft_drops,
                mv: target,
            });
        }
    }
}

fn shift(mut location: PieceLocation, collision_map: &CollisionMaps, dx: i8) -> Option<Placement> {
    location.x += dx;
    if collision_map.obstructed(location) {
        return None;
    }
    Some(Placement {
        location,
        spin: Spin::None,
    })
}

fn rotate_to(
    from: PieceLocation,
    to_rotation: Rotation,
    collision_map: &CollisionMaps,
    board: &Board,
    kickset: kicks::Kickset,
) -> Option<Placement> {
    let unkicked = PieceLocation {
        rotation: to_rotation,
        ..from
    };
    rotate(
        unkicked,
        collision_map,
        board,
        kickset
            .kicks_between(from.piece, from.rotation, to_rotation)
            .iter()
            .copied(),
    )
}

fn rotate(
    unkicked: PieceLocation,
    collision_map: &CollisionMaps,
    board: &Board,
    kicks: impl Iterator<Item = (i8, i8)>,
) -> Option<Placement> {
    for (i, (dx, dy)) in kicks.enumerate() {
        let target = PieceLocation {
            x: unkicked.x + dx,
            y: unkicked.y + dy,
            ..unkicked
        };
        if collision_map.obstructed(target) {
            continue;
        }

        let spin;
        if target.piece != Piece::T {
            spin = if non_t_spin(target, board) {
                Spin::Full
            } else {
                Spin::None
            };
        } else {
            let corners = [(-1, -1), (1, -1), (-1, 1), (1, 1)]
                .iter()
                .filter(|&&(cx, cy)| board.occupied((cx + target.x, cy + target.y)))
                .count();
            let mini_corners = [(-1, 1), (1, 1)]
                .iter()
                .map(|&c| target.rotation.rotate_cell(c))
                .filter(|&(cx, cy)| board.occupied((cx + target.x, cy + target.y)))
                .count();

            if corners < 3 {
                spin = Spin::None;
            } else if mini_corners == 2 || i == 4 {
                spin = Spin::Full;
            } else {
                spin = Spin::Mini;
            }
        }

        return Some(Placement {
            location: target,
            spin,
        });
    }

    None
}

fn non_t_spin(location: PieceLocation, board: &Board) -> bool {
    [
        PieceLocation {
            x: location.x - 1,
            ..location
        },
        PieceLocation {
            x: location.x + 1,
            ..location
        },
        PieceLocation {
            y: location.y + 1,
            ..location
        },
    ]
    .iter()
    .all(|location| location.obstructed(board))
}

#[derive(Clone, Copy, Debug, Eq)]
struct Intermediate {
    mv: Placement,
    soft_drops: u32,
}

impl PartialEq for Intermediate {
    fn eq(&self, other: &Intermediate) -> bool {
        self.soft_drops == other.soft_drops
    }
}

impl Ord for Intermediate {
    fn cmp(&self, other: &Intermediate) -> Ordering {
        self.soft_drops.cmp(&other.soft_drops)
    }
}

impl PartialOrd for Intermediate {
    fn partial_cmp(&self, other: &Intermediate) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
