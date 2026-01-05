mod audio;
mod emulator;
mod key2btn;
use crate::audio::SquareWave;
use crate::emulator::Emulator;
use crate::emulator::consts::{SCREEN_HEIGHT, SCREEN_WIDTH};

use std::process::exit;

use sdl2::audio::{AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;



fn main() {
    println!("Insert a ROM's filename (e.g., pong.ch8): ");
    let mut filename: String = String::from("./roms/");

    if let Err(err) = std::io::stdin().read_line(&mut filename) {
        print!("{err}");
        exit(1)
    }

    println!("Loading ROM: {filename}");

    let mut emulator = Emulator::new();
    if let Err(err) = emulator.load_rom(filename.trim()) {
        println!("{err}");
        exit(2);
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

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

    let audio_spec = AudioSpecDesired {
        freq: Some(44000),
        channels: Some(2),
        samples: Some(1024),
    };

    let audio_device  = audio_subsystem
        .open_playback(None, &audio_spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 150.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.05,
            }
        })
        .unwrap();

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

        emulator.tick_timers(&audio_device);

        draw_on_canvas(&mut canvas, &emulator.display);
        canvas.present();
        emulator.draw_flag = false;
    }
}

fn draw_on_canvas(
    canvas: &mut Canvas<sdl2::video::Window>,
    display: &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
) {
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
