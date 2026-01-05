pub mod consts;

use rand::Rng;
use std::{cmp::min, error::Error, fs};

use crate::{audio::AudioDeviceControl, emulator::consts::{
    FONTSET, FONTSET_START_ADDRESS, NUM_BITS_IN_BYTE, SCREEN_HEIGHT, SCREEN_WIDTH,
}};

pub struct Emulator {
    memory: [u8; 4096],
    v_registers: [u8; 16],
    index_register: u16,
    pc: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    btn_pressings: [bool; 16],
    btn_waiting_for_release: Option<u8>,
    pub display: [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT],
    pub draw_flag: bool,
}

impl Emulator {
    pub fn new() -> Emulator {
        let mut emu = Emulator {
            memory: [0; 4096],
            v_registers: [0; 16],
            index_register: 0,
            pc: 0x200,
            stack: vec![],
            delay_timer: 0,
            sound_timer: 0,
            btn_pressings: [false; 16],
            display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            draw_flag: false,
            btn_waiting_for_release: None,
        };

        emu.memory
            [consts::FONTSET_START_ADDRESS..(consts::FONTSET_START_ADDRESS + consts::FONTSET_SIZE)]
            .copy_from_slice(&FONTSET);

        emu
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let binary = fs::read(path)?;
        let ram_starting_index: u16 = 0x200;

        if binary.len() > 4096 - 0x200 {
            return Err("Binário grande demais para a memória".into());
        }

        self.memory[(ram_starting_index as usize)..(ram_starting_index as usize + binary.len())]
            .copy_from_slice(&binary);

        Ok(())
    }

    pub fn set_btn_press(&mut self, btn: u8, value: bool) {
        if btn < 16 {
            self.btn_pressings[btn as usize] = value;
        }
    }

    pub fn execution_cycle(&mut self) {
        let instruction: u16 = ((self.memory[self.pc as usize] as u16) << 8)
            | (self.memory[self.pc as usize + 1] as u16);
        self.pc += 2;
        self.execute_instruction(instruction);
    }

    pub fn execute_instruction(&mut self, instruction: u16) {
        let nibble1: u16 = (instruction & 0xF000) >> 12;
        let nibble2: u16 = (instruction & 0x0F00) >> 8;
        let nibble3: u16 = (instruction & 0x00F0) >> 4;
        let nibble4: u16 = instruction & 0x000F;

        let address_argument: u16 = instruction & 0x0FFF;
        let byte_argument: u8 = (instruction & 0x00FF) as u8;

        match (nibble1, nibble2, nibble3, nibble4) {
            (0, 0, 0xE, 0) => {
                // 00E0:
                // Clears display.
                self.display.fill([false; SCREEN_WIDTH]);
            }
            (0, 0, 0xE, 0xE) => {
                // 00EE:
                // Return from a subroutine.
                if let Some(popped) = self.stack.pop() {
                    self.pc = popped;
                } else {
                    panic!("Stack underflow")
                }
            }
            (1, _, _, _) => {
                // 1nnn:
                // Jump to address *nnn*
                self.pc = address_argument;
            }
            (2, _, _, _) => {
                // 2nnn:
                // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to *nnn*.
                self.stack.push(self.pc);
                self.pc = address_argument;
            }
            (3, _, _, _) => {
                // 3xkk:
                // Skip next instruction if Vx == kk (Vx: register number *x*).

                if self.v_registers[nibble2 as usize] == byte_argument {
                    self.pc += 2;
                }
            }
            (4, _, _, _) => {
                // 4xkk:
                // Skip next instruction if Vx != kk.

                if self.v_registers[nibble2 as usize] != byte_argument {
                    self.pc += 2;
                }
            }
            (5, _, _, 0) => {
                // 5xy0:
                // Skip next instruction if Vx == Vy.

                if self.v_registers[nibble2 as usize] == self.v_registers[nibble3 as usize] {
                    self.pc += 2;
                }
            }
            (6, _, _, _) => {
                // 6xkk:
                // The interpreter puts the value kk into register Vx.
                self.v_registers[nibble2 as usize] = byte_argument;
            }
            (7, _, _, _) => {
                // 7xkk:
                // Adds the value kk to the value of register Vx, then stores the result in Vx.
                self.v_registers[nibble2 as usize] =
                    self.v_registers[nibble2 as usize].wrapping_add(byte_argument);
            }
            (8, _, _, _) => {
                // 8xy*:
                // Performs one of a set of operations between registers Vx and Vy, defined by the last nibble.

                let x = nibble2 as usize;
                let vx_value = self.v_registers[nibble2 as usize];
                let vy_value = self.v_registers[nibble3 as usize];

                match nibble4 {
                    0 => self.v_registers[x] = vy_value,
                    1 => {
                        self.v_registers[x] = vx_value | vy_value;
                        self.v_registers[0xF] = 0
                    }
                    2 => {
                        self.v_registers[x] = vx_value & vy_value;
                        self.v_registers[0xF] = 0
                    }
                    3 => {
                        self.v_registers[x] = vx_value ^ vy_value;
                        self.v_registers[0xF] = 0
                    }
                    4 => {
                        let (sum, overflow) = vx_value.overflowing_add(vy_value);
                        self.v_registers[x] = sum;
                        self.v_registers[0xF] = if overflow { 1 } else { 0 };
                    }
                    5 => {
                        let (diff, overflow) = vx_value.overflowing_sub(vy_value);
                        self.v_registers[x] = diff;
                        self.v_registers[0xF] = if overflow { 0 } else { 1 };
                    }
                    6 => {
                        self.v_registers[x] = vy_value >> 1;
                        self.v_registers[0xF] = vx_value & 1;
                    }
                    7 => {
                        let (diff, overflow) = vy_value.overflowing_sub(vx_value);
                        self.v_registers[x] = diff;
                        self.v_registers[0xF] = if overflow { 0 } else { 1 };
                    }
                    0xE => {
                        self.v_registers[x] = vy_value << 1;
                        self.v_registers[0xF] = if (vx_value & 0x80) == 0x80 { 1 } else { 0 };
                    }
                    _ => panic!("Invalid instruction: 8xy{nibble4}"),
                }
            }
            (9, _, _, 0) => {
                // 9xy0:
                // Skip next instruction if Vx == Vy.

                if self.v_registers[nibble2 as usize] != self.v_registers[nibble3 as usize] {
                    self.pc += 2;
                }
            }
            (0xA, _, _, _) => {
                // Annn:
                // The value of register I is set to nnn.

                self.index_register = address_argument;
            }
            (0xB, _, _, _) => {
                // Bnnn:
                // The program counter is set to nnn plus the value of V0.

                self.pc = address_argument + self.v_registers[0] as u16;
            }
            (0xC, _, _, _) => {
                // Cxkk:
                // The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx.

                let random_byte: u8 = rand::rng().random_range(1..=255);

                self.v_registers[nibble2 as usize] = random_byte & byte_argument;
            }
            (0xD, _, _, _) => {
                // Dxyn:
                // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
                self.draw_flag = true;
                self.update_sprite(nibble4 as usize, nibble2 as usize, nibble3 as usize);
            }
            (0xE, _, 0x9, 0xE) => {
                // Ex9E
                // Skip next instruction if key with the value of Vx is pressed.
                let vx = self.v_registers[nibble2 as usize] as usize;
                if self.btn_pressings[vx] {
                    self.pc += 2;
                }
            }
            (0xE, _, 0xA, 0x1) => {
                // ExA1
                // Skip next instruction if key with the value of Vx is NOT pressed.
                let vx = self.v_registers[nibble2 as usize] as usize;
                if !self.btn_pressings[vx] {
                    self.pc += 2;
                }
            }
            (0xF, _, 0x0, 0x7) => {
                // Fx07
                //Set Vx = delay timer value.
                self.v_registers[nibble2 as usize] = self.delay_timer;
            }
            (0xF, _, 0x0, 0xA) => {
                // Fx0A
                // Wait for a key press, store the value of the key in Vx.

                let mut pressed = false;

                if let Some(btn) = self.btn_waiting_for_release {
                    if !self.btn_pressings[btn as usize] {
                        pressed = true;
                        self.v_registers[nibble2 as usize] = btn;
                        self.btn_waiting_for_release = None;
                    }
                } else {
                    for (i, is_btn_pressed) in self.btn_pressings.iter().enumerate() {
                        if *is_btn_pressed {
                            self.btn_waiting_for_release = Some(i as u8);
                        }
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }
            (0xF, _, 0x1, 0x5) => {
                // Fx15
                // Set delay timer = Vx.
                self.delay_timer = self.v_registers[nibble2 as usize];
            }
            (0xF, _, 0x1, 0x8) => {
                // Fx18
                // Set delay timer = Vx.
                self.sound_timer = self.v_registers[nibble2 as usize];
            }
            (0xF, _, 0x1, 0xE) => {
                // Fx1E
                // Set I = I + Vx.
                self.index_register =
                    self.index_register + self.v_registers[nibble2 as usize] as u16;
            }
            (0xF, _, 0x2, 0x9) => {
                // Fx29
                // Set I = location of sprite for digit Vx.
                let vx = self.v_registers[nibble2 as usize];
                let sprite_address = FONTSET_START_ADDRESS + (vx as usize * 5);
                self.index_register = sprite_address as u16;
            }
            (0xF, _, 0x3, 0x3) => {
                // Fx33:
                // Store Binary-Coded Decimal representation of Vx in memory locations I, I+1, and I+2.

                let index = self.index_register as usize;
                let vx = self.v_registers[nibble2 as usize];

                let hundreds = vx / 100;
                let tens = (vx % 100) / 10;
                let units = vx % 10;

                self.memory[index] = hundreds;
                self.memory[index + 1] = tens;
                self.memory[index + 2] = units;
            }
            (0xF, _, 0x5, 0x5) => {
                // Fx55
                // Store registers V0 through Vx in memory starting at location I.
                let x = nibble2;
                self.memory
                    [(self.index_register as usize)..=(self.index_register as usize + x as usize)]
                    .copy_from_slice(&self.v_registers[0..=(x as usize)]);

                self.index_register += x + 1;
            }
            (0xF, _, 0x6, 0x5) => {
                // Fx65
                // Read registers V0 through Vx from memory starting at location I.
                let x = nibble2;
                self.v_registers[0..=(x as usize)].copy_from_slice(
                    &self.memory[(self.index_register as usize)
                        ..=(self.index_register as usize + x as usize)],
                );

                self.index_register += x + 1;
            }
            _ => return,
        }
    }

    fn update_sprite(&mut self, sprite_height: usize, x: usize, y: usize) {
        let sprite = &self.memory[(self.index_register as usize)
            ..((self.index_register + sprite_height as u16) as usize)];

        let starting_x = (self.v_registers[x as usize] as usize) % SCREEN_WIDTH;
        let starting_y = (self.v_registers[y as usize] as usize) % SCREEN_HEIGHT;

        let mut must_activate_vf = false;

        let vertical_limit = min(sprite_height, SCREEN_HEIGHT - starting_y);
        let horizontal_limit = min(NUM_BITS_IN_BYTE, SCREEN_WIDTH - starting_x);

        for i in 0..vertical_limit {
            // first, we choose the **line** with *starting_y*
            let display_line = &mut self.display[starting_y + i];
            let sprite_line = sprite[i];

            for j in 0..horizontal_limit {
                let bit = 2_u8.pow((NUM_BITS_IN_BYTE - j) as u32 - 1) & sprite_line;

                if bit != 0 {
                    // then, we choose the **column** with *starting_x*
                    let line_index = starting_x + j;

                    if display_line[line_index] {
                        must_activate_vf = true;
                    }

                    display_line[line_index] = !display_line[line_index];
                }
            }
        }

        self.v_registers[0xF] = if must_activate_vf { 1 } else { 0 };
    }

    pub fn tick_timers<T: AudioDeviceControl>(&mut self, audio_device: &T) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            audio_device.resume();
            self.sound_timer -= 1;
        } else {
            audio_device.pause();
        }
    }
}
