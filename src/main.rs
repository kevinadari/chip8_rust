mod chip8;
use crate::chip8::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use std::env;
// use std::io; // Debug
use std::thread;
use std::time::Duration;

const SCALE: u32 = 10;
const SCALED_WIDTH: u32 = SCREEN_WIDTH as u32 * SCALE;
const SCALED_HEIGHT: u32 = SCREEN_HEIGHT as u32 * SCALE;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut my_chip8 = Chip8::init();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("chip8-rust", SCALED_WIDTH, SCALED_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    my_chip8.load_game(&args[1]).unwrap();
    // let mut buffer = String::new(); // Debug
    // let mut d = 1; // Debug

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }

                _ => {}
            }
        }
        // io::stdin()
            // .read_line(&mut buffer)
            // .expect("Failed to read line"); // Debug

        my_chip8.emulate();

        // println!("pc: {:02X} - {:04X}", d, my_chip8.opcode); // Debug

        if my_chip8.draw_flag {
            draw(&mut canvas, &my_chip8.screen);
            my_chip8.draw_flag = false;
        }

        thread::sleep(Duration::new(0, 100_000_000u32 / 60));
        // d += 1; // Debug
    }
}

fn draw(canvas: &mut WindowCanvas, pixels: &[u8]) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    for col in 0..SCREEN_WIDTH {
        let x: i32 = (col as u32 * SCALE) as i32;
        for row in 0..SCREEN_HEIGHT {
            let y: i32 = (row as u32 * SCALE) as i32;

            if pixels[col + row * SCREEN_WIDTH] == 0 {
                // Unset
                canvas.set_draw_color(Color::RGB(0, 0, 0));
            } else {
                // Set
                canvas.set_draw_color(Color::RGB(255, 255, 255));
            }

            canvas.fill_rect(Rect::new(x, y, SCALE, SCALE)).unwrap();
        }
    }
    canvas.present();
}
