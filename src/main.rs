mod emulator;
mod key2btn;
use crate::emulator::Emulator;

use std::process::exit;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
fn main() {
    let mut emulator = Emulator::new();

    if let Err(rom) = emulator.load_rom("./roms/alago") {
        println!("{rom}");
        exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Chip-8 Emulator", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

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

        // FDC cycle

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

#[test]
fn verify_display_update() {
    let mut emul = Emulator::new();

    emul.execute_instruction(0xA05F);
    emul.execute_instruction(0x603F);
    emul.execute_instruction(0x611F);
    emul.execute_instruction(0xD105);

    display_it(&emul.display);
}

fn display_it(display: &[[bool; 64]; 32]) {
    for line in display {
        for cell in line {
            if *cell {
                print!("O");
            } else {
                print!("-");
            }
        }
        println!("");
    }
}
