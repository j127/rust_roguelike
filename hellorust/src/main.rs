use rltk::{Rltk, GameState, Console, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{min, max};

// This tells Rust to use the macro code in the following crate.
#[macro_use]
extern crate specs_derive;

/// A position with x and y coordinates.
///
/// Note that without the derive macro, you would do:
///
/// ```rust
/// // The ECS is storing the component.
/// impl Component for Position {
///     type Storage = VecStorage<Self>;
/// }
/// ```
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

/// How to draw the entity.
#[derive(Component)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}

#[derive(Component, Debug)]
struct Player {}

/// Represents a tile type.
///
/// `PartialEq` allows the use of `==` to see if they match
/// `Clone` adds `.clone()` method.
/// `Copy` changes the default from moving to copying.
#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}

/// `World` comes from the `Specs` crate.
struct State {
    ecs: World
}

/// For entities that like moving left.
#[derive(Component)]
struct LeftMover {}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        // `fetch` will crash if the resource doesn't exist. It's a
        // `shred` type, which usually acts like a reference, but needs
        // coercing to actually become a reference.
        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        // `.join()` here only returns entities that have both
        // `Position` and `Renderable`
        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

struct LeftWalker {}

/// From [the book](http://bfnightly.bracketproductions.com/rustbook/chapter_2.html):
///
/// > Notice that this is very similar to how we wrote the rendering
/// > code - but instead of calling in to the ECS, the ECS system is
/// > calling into our function/system. It can be a tough judgment call
/// > on which to use. If your system just needs data from the ECS, then
/// > a system is the right place to put it. If it also needs access to
/// > other parts of your program, it is probably better implemented on
/// > the outside - calling in.
impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            // wrap around the screen
            if pos.x < 0 { pos.x = 79; }
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        // "tells Specs that if any changes were queued up by the
        // systems, they should apply to the world now."
        self.ecs.maintain();
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    match ctx.key {
        None => {}
        Some(key) => match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        },
    }
}

/// Find the index of the game map for x, y.
///
/// The map is a 4000-item vector. (80*50)
pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * 80) + x as usize
}

/// Create a new game map
fn new_map() -> Vec<TileType> {
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

/// Draw the map.
///
/// The tutorial author said the he passes in `&[TileType]` instead of
/// `&Vec<TileType>` in order to pass in slices of a map, if necessary.
fn draw_map(map: &[TileType], ctx: &mut Rltk) {
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

fn main() {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build();
    let mut gs = State { ecs: World::new() };

    // Tell ECS about our components
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();

    gs.ecs.insert(new_map());

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    rltk::main_loop(context, gs);
}
