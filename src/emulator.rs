use std::{error::Error, fs};

pub struct Emulator {
    memory: [u8; 4096],
    v_registers: [u8; 16],
    index_register: u16,
    pc: u16,
    stack: Vec<u16>,
    display: [bool; 64 * 32], // Representação dos pixels
    delay_timer: u8,
    sound_timer: u8,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            memory: [0; 4096],
            v_registers: [0; 16],
            index_register: 0,
            pc: 0,
            stack: vec![0],
            display: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let binary: String = fs::read_to_string(path)?;
        let binary: &[u8] = binary.as_bytes();

        let ram_starting_index: usize = 0x200;

        for i in 0..binary.len()  {
            self.memory[ram_starting_index + i] = binary[i];
        }

        self.memory[200];

        Ok(())
    }

    pub fn read_instruction(&mut self, instruction: [u8; 2]) {
        
    }
}
