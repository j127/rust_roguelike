use rltk::{Rltk, GameState, Console, RGB};
use specs::prelude::*;

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

/// For entities that like moving left.
#[derive(Component)]
struct LeftMover {}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.run_systems();

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
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);

        // "tells Specs that if any changes were queued up by the
        // systems, they should apply to the world now."
        self.ecs.maintain();
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
    gs.ecs.register::<LeftMover>();

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
            .with(LeftMover{})
            .build();
    }

    rltk::main_loop(context, gs);
}
