use raylib::prelude::*;

fn main() {
    let (mut raylib, thread) = raylib::init()
        .size(800, 600)
        .title("hello")
    .build();

    let centre_x = raylib.get_screen_width() / 2;
    let centre_y = raylib.get_screen_height() / 2;
    let example_text = "hello world!";
    let ft_size = 30;

    while !raylib.window_should_close() {
        let mut canvas = raylib.begin_drawing(&thread);
        let text_width = canvas.measure_text(example_text, ft_size);
        canvas.clear_background(Color::BLACK);
        canvas.draw_text(
            example_text,
            centre_x - text_width / 2,
            centre_y - ft_size / 2,
            ft_size,
            Color::WHITE,
        );
    }
}