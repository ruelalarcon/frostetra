use std::cmp::Ordering;
use std::collections::BinaryHeap;

use ahash::AHashMap;

use crate::tetris::model::rules::{GameRules, SonicDrop};
use crate::tetris::model::*;
use crate::tetris::movegen::collision_maps::CollisionMap;
use crate::tetris::movegen::spin::detect_spin;
use crate::tetris::movegen::MovegenBoard;

pub fn find_moves<B: MovegenBoard>(
    board: &B,
    piece: Piece,
    rules: &GameRules,
) -> Vec<(Placement, u32)> {
    puffin::profile_function!();
    let mut queue = BinaryHeap::with_capacity(64);
    let mut values = AHashMap::with_capacity(128);
    let mut underground_locks = AHashMap::with_capacity(64);
    let mut locks = Vec::with_capacity(64);
    let collision_map = board.collision_maps(piece);

    let fast_mode = rules.sonic_drop == SonicDrop::Only;
    if fast_mode {
        for &rotation in &[
            Rotation::North,
            Rotation::East,
            Rotation::South,
            Rotation::West,
        ] {
            for x in 0..board.width() {
                let mut location = PieceLocation {
                    piece,
                    rotation,
                    x: x as i8,
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
                    if let Some(mv) = rotate_to(location, rotation, &collision_map, board, rules) {
                        update_position(mv, distance as u32);
                    }
                }
                if rules.rot180 {
                    if let Some(mv) = rotate_to(
                        location,
                        location.rotation.flip(),
                        &collision_map,
                        board,
                        rules,
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
            if let Some(mv) = rotate_to(expand.mv.location, rotation, &collision_map, board, rules)
            {
                update_position(mv, expand.soft_drops);
            }
        }
        if rules.rot180 {
            if let Some(mv) = rotate_to(
                expand.mv.location,
                expand.mv.location.rotation.flip(),
                &collision_map,
                board,
                rules,
            ) {
                update_position(mv, expand.soft_drops);
            }
        }
    }

    locks.extend(underground_locks.drain().map(|(_, value)| value));
    locks.sort_by_key(|(mv, soft_drops)| (mv.sort_key(), *soft_drops));
    locks
}

fn update_position<'a>(
    queue: &'a mut BinaryHeap<Intermediate>,
    values: &'a mut AHashMap<Placement, u32>,
    fast_mode: bool,
    board: &'a impl BoardRepresentation,
) -> impl FnMut(Placement, u32) + 'a {
    move |target: Placement, soft_drops: u32| {
        if fast_mode
            && (!target.location.horizontally_in_bounds(board)
                || target.location.above_stack(board))
        {
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

fn shift(
    mut location: PieceLocation,
    collision_map: &impl CollisionMap,
    dx: i8,
) -> Option<Placement> {
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
    collision_map: &impl CollisionMap,
    board: &impl BoardRepresentation,
    rules: &GameRules,
) -> Option<Placement> {
    let unkicked = PieceLocation {
        rotation: to_rotation,
        ..from
    };
    rotate(
        unkicked,
        collision_map,
        board,
        rules,
        rules
            .kickset
            .kicks_between(from.piece, from.rotation, to_rotation)
            .iter()
            .copied(),
    )
}

fn rotate(
    unkicked: PieceLocation,
    collision_map: &impl CollisionMap,
    board: &impl BoardRepresentation,
    rules: &GameRules,
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

        return Some(Placement {
            location: target,
            spin: detect_spin(target, board, rules, i),
        });
    }

    None
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
        other.soft_drops.cmp(&self.soft_drops)
    }
}

impl PartialOrd for Intermediate {
    fn partial_cmp(&self, other: &Intermediate) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
