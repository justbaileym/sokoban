use crate::components::{Immovable, Movable, Player, Position};
use crate::constants::{MAP_HEIGHT, MAP_WIDTH};
use crate::events::{EntityMoved, Event};
use crate::resources::{EventQueue, Gameplay, InputQueue};
use ggez::event::KeyCode;
use specs::{world::Index, Entities, Join, ReadStorage, System, Write, WriteStorage};

use std::collections::HashMap;

pub struct InputSystem {}


// System implementation
impl<'a> System<'a> for InputSystem {
    // Data
    type SystemData = (
        Write<'a, EventQueue>,
        Write<'a, InputQueue>,
        Write<'a, Gameplay>,
        Entities<'a>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Movable>,
        ReadStorage<'a, Immovable>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut events,
            mut input_queue,
            mut gameplay,
            entities,
            mut positions,
            players,
            movables,
            immovables,
        ) = data;

        let mut to_move = Vec::new();

        for (position, _player) in (&positions, &players).join() {
            // Get the first key pressed
            if let Some(key) = input_queue.keys_pressed.pop_front() {
                // get all the movables and immovables
                let mov: HashMap<(u8, u8), Index> = (&entities, &movables, &positions)
                    .join()
                    .map(|t| ((t.2.x, t.2.y), t.0.id()))
                    .collect::<HashMap<_, _>>();
                let immov: HashMap<(u8, u8), Index> = (&entities, &immovables, &positions)
                    .join()
                    .map(|t| ((t.2.x, t.2.y), t.0.id()))
                    .collect::<HashMap<_, _>>();

                // Now iterate through current position to the end of the map
                // on the correct axis and check what needs to move.
                let (start, end, is_x) = match key {
                    KeyCode::W | KeyCode::Up => (position.y, 0, false),
                    KeyCode::S | KeyCode::Down => (position.y, MAP_HEIGHT, false),
                    KeyCode::A | KeyCode::Left => (position.x, 0, true),
                    KeyCode::D | KeyCode::Right => (position.x, MAP_WIDTH, true),
                    KeyCode::Q | KeyCode::Escape => std::process::exit(0),
                    _ => continue,
                };

                let range = if start < end {
                    (start..=end).collect::<Vec<_>>()
                } else {
                    (end..=start).rev().collect::<Vec<_>>()
                };

                for x_or_y in range {
                    let pos = if is_x {
                        (x_or_y, position.y)
                    } else {
                        (position.x, x_or_y)
                    };

                    // find a movable
                    // if it exists, we try to move it and continue
                    // if it doesn't exist, we continue and try to find an immovable instead
                    if let Some(id) = mov.get(&pos) {
                        to_move.push((key, *id));
                    } else {
                        if let Some(_id) = immov.get(&pos) {
                            to_move.clear();
                            events.events.push(Event::PlayerHitObstacle {});
                        }
                        break;
                    }
                }
            }
        }

        // We've just moved, so let's increase the number of moves
        if !to_move.is_empty() {
            gameplay.moves_count += 1;
        }

        // Now actually move what needs to be moved
        for (key, id) in to_move {
            let position = positions.get_mut(entities.entity(id));
            if let Some(position) = position {
                match key {
                    KeyCode::W | KeyCode::Up => position.y -= 1,
                    KeyCode::S | KeyCode::Down => position.y += 1,
                    KeyCode::A | KeyCode::Left => position.x -= 1,
                    KeyCode::D | KeyCode::Right => position.x += 1,
                    _ => (),
                }
            }

            // Fire an event for the entity that just moved
            events.events.push(Event::EntityMoved(EntityMoved { id }));
        }
    }
}
