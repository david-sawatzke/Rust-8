extern crate piston_window;
extern crate rand;

use rust_8::chip8;
use rust_8::display;
use std::env;
use std::fs::File;
use std::io::Read;

use random_fast_rng::FastRng;

use piston_window::*;

const ENLARGEMENT_FACTOR: usize = 20;
const WINDOW_DIMENSIONS: [u32; 2] = [
    (display::WIDTH * ENLARGEMENT_FACTOR) as u32,
    (display::HEIGHT * ENLARGEMENT_FACTOR) as u32,
];

fn main() {
    let file_name = env::args()
        .nth(1)
        .expect("Must give game name as first file");
    let mut file = File::open(file_name).expect("There was an issue opening the file");
    let mut game_data = Vec::new();
    file.read_to_end(&mut game_data)
        .expect("Failure to read file");

    let mut window: PistonWindow = WindowSettings::new("Rust-8 Emulator", WINDOW_DIMENSIONS)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut computer = chip8::Chip8::new(&game_data, FastRng::new());
    let mut instruction_time_left = 0.0;
    let mut clock_time_left = 0.0;
    while let Some(e) = window.next() {
        if e.render_args().is_some() {
            draw_screen(&computer.display.get_buffer(), &mut window, &e);
        }

        if let Some(u) = e.update_args() {
            instruction_time_left += u.dt;
            while instruction_time_left > 1.0 / chip8::INSTRUCTION_RATE as f64 {
                instruction_time_left -= 1.0 / chip8::INSTRUCTION_RATE as f64;
                clock_time_left += 1.0 / chip8::INSTRUCTION_RATE as f64;
                computer.run_cycle();
                if clock_time_left > 1.0 / chip8::TIMER_RATE as f64 {
                    computer.timer_tick();
                    clock_time_left -= 1.0 / chip8::TIMER_RATE as f64;
                }
            }
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            if let Some(key_value) = key_value(&key) {
                computer.handle_key_release(key_value);
            }
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if let Some(key_value) = key_value(&key) {
                computer.handle_key_press(key_value);
            }
        }
    }
}

fn key_value(key: &Key) -> Option<u8> {
    if key.code() >= 48 && key.code() <= 57 {
        Some((key.code() - 48) as u8)
    } else if key.code() >= 97 && key.code() <= 102 {
        Some((key.code() - 97 + 10) as u8)
    } else {
        None
    }
}

fn draw_screen(
    display_buffer: &display::Buffer,
    window: &mut PistonWindow,
    e: &piston_window::Event,
) {
    window.draw_2d(e, |context, graphics, _| {
        piston_window::clear(color::BLACK, graphics);

        for (i, row) in display_buffer.iter().enumerate() {
            for (j, val) in row.iter().enumerate() {
                if *val {
                    let dimensions = [
                        (j * ENLARGEMENT_FACTOR) as f64,
                        (i * ENLARGEMENT_FACTOR) as f64,
                        ENLARGEMENT_FACTOR as f64,
                        ENLARGEMENT_FACTOR as f64,
                    ];
                    Rectangle::new(color::WHITE).draw(
                        dimensions,
                        &context.draw_state,
                        context.transform,
                        graphics,
                    );
                }
            }
        }
    });
}

#[allow(dead_code)]
fn debug(display_buffer: &display::Buffer) {
    for row in display_buffer.iter() {
        print!("|");
        for val in row.iter() {
            if *val {
                print!("*")
            } else {
                print!(".")
            }
        }
        println!("|")
    }
}
