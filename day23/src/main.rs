use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
    convert::Infallible,
    fs,
    str::FromStr,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
struct State<const N: usize> {
    // indexes 0-10 = corridor. 2, 4, 6, 8 unused (rule - room entrances).
    // indexes >= 10 are rooms.
    // values: 1 - A, 2 - B, 3 - C, 4 - D.
    data: [u8; N],
}

impl<const N: usize> FromStr for State<N> {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut data: [u8; N] = [0; N];
        let room_depth = (N - 11) / 4;

        for (idx, c) in s.chars().filter(|c| ('A'..='D').contains(c)).enumerate() {
            let val = c as u8 - b'A' + 1;
            data[11 + room_depth * (idx % 4) + idx / 4] = val;
        }

        Ok(Self { data })
    }
}

impl<const N: usize> State<N> {
    const ROOM_DEPTH: usize = (N - 11) / 4;
    const VALID_HALLWAY_IDX: &'static [usize] = &[0, 1, 3, 5, 7, 9, 10];
    const COSTS: [u64; 4] = [1, 10, 100, 1000];

    fn apply(&self, step: (usize, usize, usize)) -> (Self, u64) {
        let cost = Self::COSTS[self.data[step.0] as usize - 1] * step.2 as u64;

        let mut new_data = self.data;
        new_data.swap(step.0, step.1);

        (Self { data: new_data }, cost)
    }

    fn is_complete(&self) -> bool {
        for room in 0..4 {
            let room_data =
                &self.data[11 + room * Self::ROOM_DEPTH..11 + (room + 1) * Self::ROOM_DEPTH];

            if !room_data.iter().all(|v| *v == room as u8 + 1) {
                return false;
            }
        }

        true
    }

    fn next_moves(&self) -> impl Iterator<Item = (usize, usize, usize)> {
        let mut moves = vec![];

        let room_first_idx = |room_idx| 11 + room_idx * Self::ROOM_DEPTH;
        let rooms_data = [0, 1, 2, 3].map(|i| &self.data[room_first_idx(i)..room_first_idx(i + 1)]);
        let room_depth_occupied = rooms_data.map(|room| room.iter().position(|s| *s > 0));

        // Generate from hallway to room moves.
        for valid_idx in Self::VALID_HALLWAY_IDX.iter().copied() {
            if self.data[valid_idx] > 0 {
                let valid_room = self.data[valid_idx] - 1;
                let valid_room_entrance = 2 + 2 * valid_room;
                let corridor_path = u8::min(valid_idx as u8, valid_room_entrance)
                    ..=u8::max(valid_idx as u8, valid_room_entrance);

                // We can cross through corridor without bouncing another amphipod.
                let path_unobstructed = corridor_path
                    .clone()
                    .all(|i| i as usize == valid_idx || self.data[i as usize] == 0);
                // "Move after corridor" rule is fulfilled (amphipod stopped in the corridor moves if and only if it can move to the room and room is already valid.)
                let room_ready = rooms_data[valid_room as usize]
                    .iter()
                    .copied()
                    .all(|pod| pod == 0 || pod == self.data[valid_idx]);

                if path_unobstructed && room_ready {
                    // Move amphipod immediately to maximum depth of the room.
                    let depth_to_move =
                        room_depth_occupied[valid_room as usize].unwrap_or(Self::ROOM_DEPTH) - 1;

                    moves.push((
                        valid_idx,
                        room_first_idx(valid_room as usize) + depth_to_move,
                        corridor_path.count() + depth_to_move, // we omit 1 here because we overshoot by 1 in corridor_path count!
                    ));
                }
            }
        }

        // Generate from room to hallway moves.
        for (room_idx, _) in room_depth_occupied.iter().enumerate() {
            if let Some(room_pod_depth) = room_depth_occupied[room_idx] {
                let pod = self.data[room_first_idx(room_idx) + room_pod_depth];
                let target_room = pod - 1;

                let direct_route_range = u8::min(2 + room_idx as u8 * 2, 2 + target_room * 2)
                    ..=u8::max(2 + room_idx as u8 * 2, 2 + target_room * 2);

                for valid_hallway_idx in Self::VALID_HALLWAY_IDX.iter().copied() {
                    let on_direct_route = direct_route_range.contains(&(valid_hallway_idx as u8));
                    let corridor_path = u8::min(valid_hallway_idx as u8, 2 + room_idx as u8 * 2)
                        ..=u8::max(valid_hallway_idx as u8, 2 + room_idx as u8 * 2);
                    let path_unobstructed =
                        corridor_path.clone().all(|i| self.data[i as usize] == 0);

                    // We don't want to make a move where we go to immediate room entrance if we happen to go further.
                    if !(on_direct_route && corridor_path.clone().count() > 2) && path_unobstructed
                    {
                        moves.push((
                            room_first_idx(room_idx) + room_pod_depth,
                            valid_hallway_idx,
                            corridor_path.count() + room_pod_depth,
                        ));
                    }
                }
            }
        }

        moves.into_iter()
    }
}

fn organizing_cost<const N: usize>(initial: State<N>) -> Option<u64> {
    let mut heap = BinaryHeap::from([(Reverse(0), 0, initial)]);
    let mut visited = HashMap::new();
    visited.insert(initial, 0);

    while let Some((_, cost, state)) = heap.pop() {
        if let Some(prev_cost) = visited.get(&state) {
            if *prev_cost < cost {
                continue;
            }
        }

        if state.is_complete() {
            return Some(cost);
        }

        visited.insert(state, cost);
        for possible_move in state.next_moves() {
            let (new_state, move_cost) = state.apply(possible_move);

            if (cost + move_cost) < *visited.get(&new_state).unwrap_or(&u64::MAX) {
                heap.push((Reverse(cost + move_cost), cost + move_cost, new_state));
            }
        }
    }

    None
}

fn main() {
    let state_in = fs::read_to_string("./input").unwrap();
    let state: State<{ 11 + 2 * 4 }> = state_in.parse().unwrap();

    let mut part2_input = state_in.lines().collect::<Vec<_>>();
    part2_input.splice(3..3, ["#D#C#B#A#", "#D#B#A#C#"]);

    let state_part2: State<{ 11 + 4 * 4 }> = part2_input.join("\n").parse().unwrap();

    if let Some(cost) = organizing_cost(state) {
        println!("Smallest cost for organizing amphipods is {}", cost);
    } else {
        println!("Couldn't find solution for given data.");
    }

    if let Some(cost_part2) = organizing_cost(state_part2) {
        println!(
            "Smallest cost for organizing amphipods after unfolding is: {}",
            cost_part2
        );
    } else {
        println!("Couldn't find solution for given data (after unfolding).");
    }
}
