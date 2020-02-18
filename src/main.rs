use specs::prelude::*;
// This tells Rust to use the macro code in the following crate.
#[macro_use]
extern crate specs_derive;

use rltk::{ Console, GameState, Rltk, RGB };
mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
use rect::Rect;

/// `World` comes from the `Specs` crate.
pub struct State {
    pub ecs: World
}

impl State {
    fn run_systems(&mut self) {
        // "tells Specs that if any changes were queued up by the
        // systems, they should apply to the world now."
        self.ecs.maintain();
    }
}


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

    let (rooms, map) = new_map_rooms_and_corridors();
    gs.ecs.insert(map);
    let (player_x, player_y) = rooms[0].center();

    gs.ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    rltk::main_loop(context, gs);
}
