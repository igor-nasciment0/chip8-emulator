mod emulator;
mod key2btn;
use crate::emulator::Emulator;
use crate::emulator::consts::{SCREEN_WIDTH, SCREEN_HEIGHT};

use std::process::exit;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;

fn main() {
    let mut emulator = Emulator::new();

    if let Err(rom) = emulator.load_rom("./roms/6-keypad.ch8") {
        println!("{rom}");
        exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "Chip-8 Emulator",
            SCREEN_WIDTH as u32 * 10,
            SCREEN_HEIGHT as u32 * 10,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas: Canvas<sdl2::video::Window> =
        window.into_canvas().present_vsync().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    const INSTRUCTIONS_PER_FRAME: usize = 20;

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

                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(btn) = key2btn::key2btn(key) {
                        emulator.set_btn_press(btn, true);
                    }
                }

                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(btn) = key2btn::key2btn(key) {
                        emulator.set_btn_press(btn, false);
                    }
                }
                _ => {}
            }
        }

        // Fetch, Decode, Execute Cycle

        for _ in 0..INSTRUCTIONS_PER_FRAME {
            emulator.execution_cycle();

            if emulator.draw_flag {
                break;
            }
        }

        emulator.tick_timers();

        draw_on_canvas(&mut canvas, &emulator.display);
        canvas.present();
        emulator.draw_flag = false;
    }
}

fn draw_on_canvas(canvas: &mut Canvas<sdl2::video::Window>, display: &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
    for (y, line) in display.iter().enumerate() {
        for (x, pixel) in line.iter().enumerate() {
            if *pixel {
                canvas.set_draw_color(Color::RGB(255, 255, 255));
            } else {
                canvas.set_draw_color(Color::RGB(0, 0, 0));
            }

            let _ = canvas.fill_rect(Rect::new(x as i32 * 10, y as i32 * 10, 10, 10));
        }
    }
}