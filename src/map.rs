use rltk::{ RGB, Rltk, Console, RandomNumberGenerator };
use super::{ Rect };
use std::cmp::{ min, max };

/// Represents a tile type.
///
/// `PartialEq` allows the use of `==` to see if they match
/// `Clone` adds `.clone()` method.
/// `Copy` changes the default from moving to copying.
#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall, Floor
}

/// Find the index of the game map for x, y.
///
/// The map is a 4000-item vector. (80*50)
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

/// Create a new game map with solid boundaries and 400 randomly placed
/// walls.
pub fn new_map_test() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; 80*50];

    // Make the boundary walls
    for x in 0..80 {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, 49)] = TileType::Wall;
    }

    for y in 0..50 {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(79, y)] = TileType::Wall;
    }

    // Randomly splat a bunch of walls.
    let mut rng = rltk::RandomNumberGenerator::new();

    for _i in 0..400 {
        // roll 1d79
        let x = rng.roll_dice(1, 79);
        let y = rng.roll_dice(1, 49);
        let idx = xy_idx(x, y);
        if idx != xy_idx(40, 25) {
            map[idx] = TileType::Wall;
        }
    }

    map
}

/// Set all the points on the map to Floor tiles.
fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    // `..=` means an inclusive range
    for y in room.y1 + 1 ..= room.y2 {
        for x in room.x1 + 1 ..= room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2) ..= max(x1, x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < 80 * 50 {
            map[idx as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2) ..= max(y1, y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < 80 * 50 {
            map[idx as usize] = TileType::Floor;
        }
    }
}

pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; 80*50];

    let mut rooms: Vec<Rect> = Vec::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for _i in 0..MAX_ROOMS {
        let w =  rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, 80 - w - 1) - 1;
        let y = rng.roll_dice(1, 50 - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in rooms.iter() {
            if new_room.intersect(other_room) { ok = false }
        }
        if ok {
            apply_room_to_map(&new_room, &mut map);

            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut map, prev_x, new_x, prev_y);
                    apply_horizontal_tunnel(&mut map, prev_y, prev_y, new_x);
                } else {
                    apply_vertical_tunnel(&mut map, prev_y, new_y, prev_x);
                    apply_vertical_tunnel(&mut map, prev_x, prev_x, new_y);
                }
            }

            rooms.push(new_room);
        }
    }
    (rooms, map)
}

/// Draw the map.
///
/// The tutorial author said the he passes in `&[TileType]` instead of
/// `&Vec<TileType>` in order to pass in slices of a map, if necessary.
pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        match tile {
            TileType::Floor => {
                // `to_cp437` converts unicode to DOX/CP437 char set. (☺' is 1.)
                // http://dwarffortresswiki.org/index.php/Character_table
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5),
                              RGB::from_f32(0., 0., 0.),
                              rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0),
                              RGB::from_f32(0., 0., 0.),
                              rltk::to_cp437('#'));
            }
        }

        // move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

