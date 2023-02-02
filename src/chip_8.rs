use std::fs::File;
use std::io::Read;
use macroquad::prelude::*;

pub const QUAD: f32 = 15.0;
const MEM_SIZE: usize = 0x1000;
const DISPLAY_SIZE: usize = 64 * 32;

const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip {
    mem: [u8; MEM_SIZE],
    display: [u8; DISPLAY_SIZE],
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    pub dt: u8,
    pub st: u8,
    regist: [u8; 0x10],
    game_size: u16,
    allow_draw: bool
}

impl Chip {
    pub fn new() -> Self {
        let mut mem = [0; MEM_SIZE];
        for i in 0..FONT.len() {
            mem[i + 0x50] = FONT[i]; 
        }
        Self {
            mem,
            display:[0; 64 * 32],
            pc: 0x200,
            i: 0,
            stack: vec![],
            dt: 0,
            st: 0,
            regist: [0; 0x10],
            game_size: 0,
            allow_draw: false
        }
    }

    pub fn load_game(&mut self) {
        let mut file = File::open("data/breakout.ch8").expect("file not find!");
        let data_size = std::fs::metadata("data/breakout.ch8").expect("file not find!");
        let mut buffer = vec![0; data_size.len() as usize];
        file.read(&mut buffer).expect("buffer overflow");
        for i in  0..buffer.len() {
            self.mem[i + 0x200] = buffer[i];
        }
        self.game_size = buffer.len() as u16 + 0x200;
         
    }

    pub fn emular(&mut self) {
        let opcode = (self.mem[self.pc as usize] as u16) << 8 | self.mem[(self.pc + 1) as usize] as u16;
        let p: u16 = (opcode & 0xF000) >> 12;
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = opcode as u8 & 0x000F;
        let nn = opcode as u8 & 0x00FF;
        let nnn: u16 = opcode & 0x0FFF;
        let vx = self.regist[x as usize];
        let vy = self.regist[y as usize];
        println!("{}: {:04x}", self.pc, opcode);
        match (p, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => self.display = [0;64 * 32],
            (0x0, 0x0, 0xE, 0xE) => self.pc = self.stack.pop().expect("vetor vazio") - 2,
            (0x1, ..) => self.pc = nnn - 2,
            (0x2, ..) => {
                self.stack.push(self.pc + 2);
                self.pc = nnn - 2;
            },
            (0x3, ..) => if vx == nn {self.pc += 2},
            (0x4, ..) => if vx != nn {self.pc += 2},
            (0x5, ..) => if vx == vy {self.pc += 2},
            (0x6, ..) => self.regist[x as usize] = nn,
            (0x7, ..) => {
                let result = vx as u16 + nn as u16;
                self.regist[x as usize] = result as u8;
            },
            (0x8, .., 0x0) => self.regist[x as usize] = vy,
            (0x8, .., 0x1) => self.regist[x as usize] |= vy,
            (0x8, .., 0x2) => self.regist[x as usize] &= vy,
            (0x8, .., 0x3) => self.regist[x as usize] ^= vy,
            (0x8, .., 0x4) => {
                let result = vx as u16 + vy as u16;
                self.regist[0xF] = if result > 0xFF {1} else {0};
                self.regist[x as usize] = result as u8;
                
            },
            (0x8, .., 0x5) => {
                self.regist[0xF] = if vx > vy {1} else {0};
                let result = vx as i16 - vy as i16;
                self.regist[x as usize] = result as u8;
            },
            (0x8, .., 0x6) => {
                self.regist[0xF] = 1 & vx;
                self.regist[x as usize] >>=1
            },
            (0x8, .., 0x7) => {
                self.regist[0xF] = if vy > vx {1} else {0};
                let result = vy as i16 - vx as i16;
                self.regist[x as usize] = result as u8;
            },
            (0x8, .., 0xE) => {
                self.regist[0xF] = (0b10000000 & vx) >> 7;
                self.regist[x as usize] <<= 1; 
            },
            (0x9, ..) => if vx != vy {self.pc += 2},
            (0xA, ..) => self.i = nnn,
            (0xB, ..) => self.pc = self.regist[0] as u16 + nnn - 2,
            (0xC, ..) => self.regist[x as usize] = rand::gen_range(0,256) as u8 & nn,
            (0xD, ..) => {
                for h in 0..n as u16 {
                    let sprite = self.mem[(h + self.i) as usize];
                    self.regist[0xF] = 0;
                    for cod_x in 0..8 {
                        if sprite & (0x80 >> cod_x) != 0 {
                            let pixel = ((vy as u16 + h) * 64 + vx as u16 + cod_x) as usize;
                            self.display[pixel] ^= 1;
                            if self.display[pixel] == 0 {
                                self.regist[0xF] = 1;
                            }
                        }
                    }
                }
                self.allow_draw = true;
            },
            (0xE, _, 0x9, 0xE) => if is_key(self.regist[x as usize]) {self.pc += 2},
            (0xE, _, 0xA, 0x1) => if !is_key(self.regist[x as usize]) {self.pc += 2},
            (0xF, _, 0x0, 0x7) => self.regist[x as usize] = self.dt,
            (0xF, _, 0x0, 0xA) => if let Some(key) = input() {self.regist[x as usize] = key} else {self.pc -=2},
            (0xF, _, 0x1, 0x5) => self.dt = vx,
            (0xF, _, 0x1, 0x8) => self.st = vx,
            (0xF, _, 0x1, 0xE) => self.i += self.regist[x as usize] as u16,
            (0xF, _, 0x2, 0x9) => self.i = 0x50 + vx as u16 * 5,
            (0xF, _, 0x3, 0x3) => {
                self.mem[self.i as usize] = vx / 100;
                self.mem[self.i as usize + 1] = (vx % 100) / 10;
                self.mem[self.i as usize + 2] = vx % 10;
            },
            (0xF, _, 0x5, 0x5) => {
                for c in 0..=x as u16 {
                    self.mem[(self.i + c) as usize] = self.regist[c as usize];
                }
            },
            (0xF, _, 0x6, 0x5) => {
                for c in 0..=x as u16 {
                    self.regist[c as usize] = self.mem[(self.i + c) as usize];
                }
            },
            _ => (),
        }
        self.pc += 2;
        //self.pc = if self.pc + 2 < self.game_size {self.pc + 2} else {0x200};
    }

    pub fn render(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                draw_rectangle(x as f32 * QUAD, y as f32 * QUAD, QUAD, QUAD, if self.display[y * 64 + x] == 0 { BLACK } else { WHITE }) 
            }
        }
        self.allow_draw = false;
    }
    pub fn ciclo(&mut self) {
        while !self.allow_draw {
            self.emular();
        }
        self.render();
    } 
}

fn is_key(key: u8) -> bool {
    let key = match key {
        0x1 => KeyCode::Key1,
        0x2 => KeyCode::Key2,
        0x3 => KeyCode::Key3,
        0xC => KeyCode::Key4,
        0x4 => KeyCode::Q,
        0x5 => KeyCode::W,
        0x6 => KeyCode::E,
        0xD => KeyCode::R,
        0x7 => KeyCode::A,
        0x8 => KeyCode::S,
        0x9 => KeyCode::D,
        0xE => KeyCode::F,
        0xA => KeyCode::Z,
        0x0 => KeyCode::X,
        0xB => KeyCode::C,
        0xF => KeyCode::V,
        _ => panic!("keycode")
    };
    is_key_down(key)
}

fn input() -> Option<u8> {
    let key_pressed = match get_last_key_pressed() {
        Some(key) => match key {
            KeyCode::Key1 => 0x1,
            KeyCode::Key2 => 0x2,
            KeyCode::Key3 => 0x3,
            KeyCode::Key4 => 0xC,
            KeyCode::Q => 0x4,
            KeyCode::W => 0x5,
            KeyCode::E => 0x6, 
            KeyCode::R => 0xD,
            KeyCode::A => 0x7,
            KeyCode::S => 0x8,
            KeyCode::D => 0x9,
            KeyCode::F => 0xE,
            KeyCode::Z => 0xA,
            KeyCode::X => 0x0,
            KeyCode::C => 0xB,
            KeyCode::V => 0xF,
            _ => return None,
        },
        _ => return None,
    };
    Some(key_pressed)
}

// 6
// a
// d
// 7
// 3
// 1
