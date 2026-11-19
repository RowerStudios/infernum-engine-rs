use raylib::prelude::*;

mod modules {
    pub mod mainmenu;
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(800, 600)
        .title("hello")
        .build();

    while !rl.window_should_close() {
        modules::mainmenu::display(&mut rl, &thread);
    }
}