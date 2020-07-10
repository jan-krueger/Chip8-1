use std::fs::File;
use std::io::Read;
use crate::chip8::ProgramCounter::{JUMP, SKIP, NEXT};
use rand::prelude::*;

const OPCODE_SIZE: usize = 2;

pub(crate) struct Chip8 {
    registers   : [u8;16],
    memory      : [u8;4096],
    index       : usize,
    pc          : usize,
    stack       : [usize;16],
    sp          : usize,
    delay_timer : u8,
    sound_timer : u8,
    video       : [[u8; 64]; 32],

    keypad      : [bool;16],
    keypad_reg  : u8,
    keypad_waiting: bool,
}

pub struct Result {
    pub video_changed : bool,
}

impl Chip8 {

    const START_ADDRESS : usize = 0x200;
    const FONT : [u8;16*5] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0,
        0x20, 0x60, 0x20, 0x20, 0x70,
        0xF0, 0x10, 0xF0, 0x80, 0xF0,
        0xF0, 0x10, 0xF0, 0x10, 0xF0,
        0x90, 0x90, 0xF0, 0x10, 0x10,
        0xF0, 0x80, 0xF0, 0x10, 0xF0,
        0xF0, 0x80, 0xF0, 0x90, 0xF0,
        0xF0, 0x10, 0x20, 0x40, 0x40,
        0xF0, 0x90, 0xF0, 0x90, 0xF0,
        0xF0, 0x90, 0xF0, 0x10, 0xF0,
        0xF0, 0x90, 0xF0, 0x90, 0x90,
        0xE0, 0x90, 0xE0, 0x90, 0xE0,
        0xF0, 0x80, 0x80, 0x80, 0xF0,
        0xE0, 0x90, 0x90, 0x90, 0xE0,
        0xF0, 0x80, 0xF0, 0x80, 0xF0,
        0xF0, 0x80, 0xF0, 0x80, 0x80
    ];
    const SCREEN_WIDTH : usize = 64;
    const SCREEN_HEIGHT : usize = 32;
    const DEBUG : bool = true;

    pub fn new() -> Chip8 {
        let mut c = Chip8 {
            registers: [0;16],
            memory: [0;4096],
            index: 0,
            pc: Chip8::START_ADDRESS,
            stack: [0;16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            video: [[0; 64]; 32],

            keypad: [false; 16],
            keypad_reg: 0,
            keypad_waiting: false,
        };
        for i in 0..Chip8::FONT.len() {
            c.memory[i] = Chip8::FONT[i];
        }
        return c;
    }

    pub fn get_pixel(&self, pixel: i32) -> u8
    {
        let x : usize = (pixel % Chip8::SCREEN_WIDTH as i32) as usize;
        let y : usize = (pixel / Chip8::SCREEN_WIDTH as i32) as usize;

        return self.video[y][x];
    }

    pub fn get_pixelc(&self, x: usize, y: usize) -> u8
    {
        return self.video[y][x];
    }

    pub fn load_rom(&mut self)
    {
        let mut file = File::open("airplane.ch8").expect("Failed to open file.");
        let mut buffer : [u8;3584] = [0;3584];
        let length = file.read(&mut buffer).expect("Failed to read to buffer.");

        if length > 3584 {
            panic!("The ROM is too big!");
        }

        self.memory[Chip8::START_ADDRESS..].copy_from_slice(&buffer);
    }

    pub fn fetch_instruction(&self) -> u16
    {
        let opcode = (self.memory[self.pc] as u16) << 8 | self.memory[self.pc + 1] as u16;
        return opcode;
    }

    pub fn log(message: String) {
        if Chip8::DEBUG {
            print!("{}", message);
        }
    }

    pub fn execute_instruction(&mut self, keys_pressed: &[bool; 16]) -> Result
    {
        let mut video_changed = false;
        let opcode = self.fetch_instruction();

        Chip8::log(format!("{:#X} -> fetching opcode: {:#X}", self.pc, opcode));
        let a = ((opcode & 0xF000) >> 12) as u8;
        let b = ((opcode & 0x0F00) >> 8) as u8;
        let c = ((opcode & 0x00F0) >> 4) as u8;
        let d = ((opcode & 0x000F) >> 0) as u8;

        let addr = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = b as usize;
        let y = c as usize;
        let nibble = d as usize;

        if self.keypad_waiting {
            for i in 0..keys_pressed.len() {
                if keys_pressed[i] {
                    self.keypad_waiting = false;
                    self.registers[self.keypad_reg as usize] = i as u8;
                    break;
                }
            }
            println!();
            return Result { video_changed };
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        let action : ProgramCounter = match (a, b, c, d) {
            // clear display
            (0x0, 0x0, 0xE, 0x0) => {
                for y in 0..Chip8::SCREEN_HEIGHT {
                    for x in 0..Chip8::SCREEN_WIDTH {
                        self.video[y][x] = 0;
                    }
                }
                video_changed = true;
                Chip8::log(format!(" -> clear display"));
                ProgramCounter::NEXT
            },
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp -= 1;
                let return_address = self.stack[self.sp];
                Chip8::log(format!(" -> return from subroutine from {:#X} to {:#X}", self.pc, return_address));
                ProgramCounter::JUMP(return_address)
            },
            // Jumps to address NNN.
            (0x1, _, _, _) => {
                Chip8::log(format!(" -> jump"));
                JUMP(addr)
            },
            // Calls subroutine at NNN.
            (0x2, _, _, _) => {
                Chip8::log(format!(" -> call subroutine from {:#X} to {:#X}", self.pc, addr));
                self.stack[self.sp] = self.pc + OPCODE_SIZE;
                self.sp += 1;
                ProgramCounter::JUMP(addr)
            },
            // Skips the next instruction if VX equals NN.
            (0x3, _, _, _) => {
                Chip8::log(format!(" -> skip (Vx == constant)"));
                ProgramCounter::skip(self.registers[x] == kk)
            },
            // Skips the next instruction if VX doesn't equal NN.
            (0x4, _, _, _) => {
                Chip8::log(format!(" -> skip (Vx != constant)"));
                ProgramCounter::skip(self.registers[x] != kk)

            },
            (0x5, _, _, 0x0) => {
                Chip8::log(format!(" -> skip (Vx == Vy)"));
                ProgramCounter::skip(self.registers[x] == self.registers[y] )
            } // Skips the next instruction if VX equals VY.
            (0x6, _, _, _) => {
                Chip8::log(format!(" -> Vx = constant"));
                self.registers[x] = kk;
                ProgramCounter::NEXT
            },
            (0x7, _, _, _) => {
                Chip8::log(format!(" -> Vx += constant"));
                let a = self.registers[x] as u16;
                let b = kk as u16;
                self.registers[x] = (a + b) as u8;
                ProgramCounter::NEXT
            },
            (0x8, _, _, 0x0) => {
                Chip8::log(format!(" -> Vx = Vy"));
                self.registers[x] = self.registers[y];
                ProgramCounter::NEXT
            },
            (0x8, _, _, 0x1) => {
                Chip8::log(format!(" -> Vx |= Vy"));
                self.registers[x] |= self.registers[y];
                ProgramCounter::NEXT
            },
            (0x8, _, _, 0x2) => {
                Chip8::log(format!(" -> Vx &= Vy"));
                self.registers[x] &= self.registers[y];
                ProgramCounter::NEXT
            },
            (0x8, _, _, 0x3) => {
                print!(" -> Vx ^= Vy");
                self.registers[x] ^= self.registers[y];
                ProgramCounter::NEXT
            },
            (0x8, _, _, 0x4) => {
                Chip8::log(format!(" -> Vx + Vy"));
                let r = (self.registers[x] as u16) + (self.registers[y] as u16);
                self.registers[x] = r as u8;
                // set carry bit
                if r > 0xFF {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                ProgramCounter::NEXT
            },
            (0x8, _, _, 0x5) => {

                if self.registers[x] > self.registers[y] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
                ProgramCounter::NEXT
            },
            (0x8, _, 0, 0x6) => {
                self.registers[0xF] = self.registers[x] & 0x1;
                self.registers[x] >>= 1;
                ProgramCounter::NEXT
            },
            (0x8, _, _, 0x7) => {
                if self.registers[y] > self.registers[x] {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
                NEXT
            },
            (0x8, _, _, 0xE) => {
                self.registers[0xF] = self.registers[x] & 0x80;
                self.registers[x] <<= 1;
                NEXT
            },
            (0x9, _, _, 0x0) => {
                Chip8::log(format!(" -> skip if(Vx!=Vy)"));
                ProgramCounter::skip(self.registers[x] != self.registers[y])
            }
            (0xA, _, _, _) => {
                Chip8::log(format!(" -> set I = addr"));
                self.index = addr as usize;
                ProgramCounter::NEXT
            },
            (0xB, _, _, _) => {
                Chip8::log(format!(" -> jump to V0+addr"));
                self.pc = (self.registers[0] as usize) + kk as usize;
                ProgramCounter::NEXT
            },
            (0xC, _, _, _) => {
                Chip8::log(format!(" -> rnd()&kk"));
                self.registers[x] = rand::thread_rng().gen::<u8>() & kk;
                ProgramCounter::NEXT
            },
            (0xD, _, _, _) => {
                Chip8::log(format!(" -> draw call"));
                self.registers[0xF] = 0;
                for byte in 0..nibble {
                    let y = (self.registers[y] as usize + byte) % Chip8::SCREEN_HEIGHT;
                    for bit in 0..8 {
                        let x = (self.registers[x] as usize + bit) % Chip8::SCREEN_WIDTH;
                        let color = (self.memory[self.index + byte] >> (7 - bit) as u8) & 1;
                        self.registers[0xF] |= color & self.video[y][x];
                        self.video[y][x] ^= color as u8;
                    }
                }

                video_changed = true;

                NEXT
            },
            (0xE, _, 0x9, 0xE) => {
                Chip8::log(format!(" -> key pressed {:#X}", self.registers[x]));
                ProgramCounter::skip(keys_pressed[self.registers[x] as usize])
            },
            (0xE, _, 0xA, 0x1) => {
                Chip8::log(format!(" -> key not pressed {:#X}", self.registers[x]));
                ProgramCounter::skip(!keys_pressed[self.registers[x] as usize])
            },
            (0xF, _, 0x0, 0x7) => {
                Chip8::log(format!(" -> Vx = delay_timer"));
                self.registers[x] = self.delay_timer;
                NEXT
            }
            (0xF, _, 0x0, 0xA) => {
                Chip8::log(format!(" -> keypad waiting"));
                self.keypad_waiting = true;
                NEXT
            },
            (0xF, _, 0x1, 0x5) => {
                Chip8::log(format!(" -> delay_timer = Vx"));
                self.delay_timer = self.registers[x];
                NEXT
            },
            (0xF, _, 0x1, 0x8) => {
                Chip8::log(format!(" -> sound_timer = Vx"));
                self.sound_timer = self.registers[x];
                NEXT
            },
            (0xF, _, 0x1, 0xE) => {
                Chip8::log(format!(" -> index + Vx"));
                self.index += self.registers[x] as usize;
                if self.index > 0x0F00 {
                    self.registers[0xF] = 1;
                } else {
                    self.registers[0xF] = 0;
                }
                NEXT
            },
            (0xF, _, 0x2, 0x9) => {
                self.index = (self.registers[x] as usize) * 5;
                NEXT
            },
            (0xF, _, 0x5, 0x5) => {
                for i in 0..x + 1 {
                    self.memory[self.index + i] = self.registers[i];
                }
                NEXT
            },
            (0xF, _, 0x6, 0x5) => {
                for i in 0..x + 1 {
                    self.registers[i] = self.memory[self.index + i];
                }
                NEXT
            },
            (0xF, _, 0x3, 0x3) => {
                self.memory[self.index + 0] = self.registers[x] / 100;
                self.memory[self.index + 1] = (self.registers[x] % 100) / 10;
                self.memory[self.index + 2] = self.registers[x] % 10;
                NEXT
            },
            _ => {
                panic!("Unimplemented opcode {:#X}", opcode);
                NEXT
            }
        };

        match action {
            NEXT => {
                self.pc += 2;
            },
            SKIP => {
                self.pc += 4;
            },
            JUMP(jump_address) => {
                self.pc = jump_address;
            },
        }

        Chip8::log("\n".parse().unwrap());
        return Result { video_changed, }
    }

}

enum ProgramCounter {
    NEXT,
    SKIP,
    JUMP(usize),
}

impl ProgramCounter {
    pub fn skip(expression: bool) -> ProgramCounter {
        if expression { SKIP } else { NEXT }
    }
}
