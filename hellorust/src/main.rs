use rltk::{Rltk, GameState, Console, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

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

/// `World` comes from the `Specs` crate.
struct State {
    ecs: World
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        // `.join()` here is like a DB join in that it only returns
        // `entities that have both Position` and `Renderable`
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

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .build();

    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                // `to_cp437` converts unicode to DOX/CP437 char set. (☺' is 1.)
                // http://dwarffortresswiki.org/index.php/Character_table
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .build();
    }

    rltk::main_loop(context, gs);
}
