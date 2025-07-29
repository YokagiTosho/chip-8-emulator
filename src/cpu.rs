use std::process;
use rand::{thread_rng, Rng};

use crate::consts::*;

enum InstructionOrd {
    Next,
    Skip,
    Jump(u16),
}

pub struct Cpu {
    v: [u8; REGISTER_COUNT],
    i: u16,
    pc: u16, 
    sp: u8,
    stack: [u16; STACK_SIZE],
    mem: [u8; RAM_SIZE],

    dt: u8, // delay timer
    st: u8, // sound timer

    keys: [bool; 16],
    key_waiting: bool, 
    key_to_store: Option<usize>,

    pub vmem: [[u8; SCR_HEIGHT]; SCR_WIDTH],
    pub vmem_changed: bool,
    
    cycle: usize,
}

impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\tRegisters: {}\n\t\
                     I: 0x{:x}\n\t\
                     Program Counter: 0x{:x}\n\t\
                     Delay Timer: {}\n\t\
                     Sound Timer: {}\n",
            self.format_registers(), self.i, self.pc, self.dt, self.st)
    }
}

impl Cpu {

    fn trace_instructions(&self) {
        //println!("{:?}", self.mem[(self.pc-10) as usize..(self.pc+10) as usize]);
        let mut row = 0;
        for i in 
            self.mem[(self.pc-10) as usize..(self.pc+10) as usize].iter()
        {
            print!("0x{:x} ", *i);
            row += 1;
            if row == 6 {
                println!("");
                row = 0;
            }
        }
    }


    fn format_registers(&self) -> String {
        format!("[V[0x0]:({}), \
                 V[0x1]:({}), \
                 V[0x2]:({}), \
                 V[0x3]:({}), \
                 V[0x4]:({}), \
                 V[0x5]:({}), \
                 V[0x6]:({}), \
                 V[0x7]:({}), \
                 V[0x8]:({}), \
                 V[0x9]:({}), \
                 V[0xA]:({}), \
                 V[0xB]:({}), \
                 V[0xC]:({}), \
                 V[0xD]:({}), \
                 V[0xE]:({}), \
                 V[0xF]:({})]",
                 self.v[0x0], self.v[0x1], self.v[0x2], self.v[0x3],
                 self.v[0x4], self.v[0x5], self.v[0x6], self.v[0x7],
                 self.v[0x8], self.v[0x9], self.v[0xA], self.v[0xB],
                 self.v[0xC], self.v[0xD], self.v[0xE], self.v[0xF])
    }

    // loading fonts in first 80 bytes of memory
    fn load_fonts(mem: &mut [u8]) { 
        for i in 0..16 {
            for j in 0..5 {
                mem[i*5+j] = FONTS[i][j];
            }
        }
    }

    pub fn new(file: Vec<u8>) -> Self {
        if file.len() > RAM_SIZE && file.len() == 0 {
            eprintln!("Can't read chip-8 file because its too large\
                either its 0 size");
            process::exit(1);
        }
        // Most Chip-8 programs start at location 0x200 
        let mut memory: [u8; RAM_SIZE] = [0; RAM_SIZE];

        for (i, val) in file.iter().enumerate() {
            memory[0x200+i] = *val;
        }

        Cpu::load_fonts(&mut memory);

        Self {
            // cpu and mem
            v: [0; REGISTER_COUNT],
            i: 0,
            pc: START_ADDR,
            sp: 0,
            stack: [0; STACK_SIZE],
            mem: memory,

            // timers
            dt: 0,
            st: 0,

            // keyboard
            keys: [false; 16],
            key_waiting: false,
            key_to_store: None,

            //video,
            vmem: [[0x0; SCR_HEIGHT]; SCR_WIDTH],
            vmem_changed: false,

            cycle: 0,
        }
    }
    
    // stack is used for stack frames
    fn push(&mut self) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc + 2;
        //println!("push sp: {}", self.sp);
    }

    fn pop(&mut self) -> u16 {
        let n = self.stack[self.sp as usize];
        self.sp -= 1;
        //println!("pop sp: {}", self.sp);
        n
    }

    pub fn tick(&mut self, keys: [bool; 16]) {
        self.cycle += 1;
        self.keys = keys;
        self.vmem_changed = false;

        if self.key_waiting {
            for (i, key) in self.keys.iter().enumerate() {
                if *key {
                    match self.key_to_store {
                        Some(t) => self.v[t] = i as u8,
                        None => panic!("Uncreachable"),
                    }
                    self.key_waiting = false;
                }
            }
        } else {
            if self.cycle % 12 == 0 {
                if self.dt > 0 {
                    self.dt -= 1;
                }
                if self.st > 0 {
                    self.st -= 1;
                }
            }
            //self.trace_instructions();
            let op = self.read_next_instruction();
            self.instruction_parser(op);
        }
    }
    
    fn read_next_instruction(&mut self) -> u16 {
        u16::from_be_bytes(
            [self.mem[self.pc as usize], self.mem[self.pc as usize+1]]
        )
    }

    fn instruction_parser(&mut self, op: u16) {
        let x = ((op << 4) >> 12) as usize;
        let y = ((op << 8) >> 12) as usize;
        let kk = ((op << 8) >> 8) as u8;
        let n = ((op << 12) >> 12) as u8;
        let nnn = ((op << 4) >> 4) as u16;

        let programm_counter = match op >> 12 {
            0x0  => {
                if op == 0x00e0 { // CLR_SCR
                    self.i_00e0()
                } else if op == 0x00ee { // RET
                    self.i_00ee()
                } else {
                    // ignore this: This instruction is only used on the old computers on which
                    // Chip-8 was originally implemented. It is ignored by modern interpreters. 
                    self.i_0nnn()
                }
            },
            // jump to addr(nnn)
            0x1 => self.i_1nnn(nnn),
            // call addr
            0x2  => self.i_2nnn(nnn),
            // Skip next instruction if Vx = kk.
            0x3  => self.i_3xkk(x, kk),
            // Skip next instruction if Vx != kk.
            0x4  => self.i_4xkk(x, kk),
            // Skip next instruction if Vx == Vy.
            0x5  => self.i_5xy0(x, y),
            // put the value into register. Set Vx = kk.
            0x6  => self.i_6xkk(x, kk),
            // ADD Vx, byte
            0x7  => self.i_7xkk(x, kk),
            // Operations with Vx and Vy registers
            0x8  => {
                // match least significant 4 bits
                match n {
                    // Stores the value of register Vy in register Vx.
                    0x0 => self.i_8xy0(x, y),
                    // OR V[x] with V[y]
                    0x1 => self.i_8xy1(x, y),
                    // AND V[x] with V[y]
                    0x2 => self.i_8xy2(x, y),
                    // XOR V[x] with V[y]
                    0x3 => self.i_8xy3(x, y),
                    // ADD Vx, Vy
                    0x4 => self.i_8xy4(x, y),
                    // SUB Vx, Vy
                    0x5 => self.i_8xy5(x, y),
                    // SHR Vx {, Vy}
                    0x6 => self.i_8xy6(x, y),
                    // SUBN Vx, Vy
                    0x7 => self.i_8xy7(x, y),
                    // SHL Vx {, Vy}
                    0xE => self.i_8xye(x, y),
                    e => panic!("undefined LS 4 bits: {} in 0x8", e),
                }
            },
            // Skip next instruction if Vx != Vy. 
            0x9  => self.i_9xy0(x, y),
            // Set I = nnn.
            0xA => self.i_annn(nnn),
            // Jump to location nnn + V0.
            0xB => self.i_bnnn(nnn),
            // Set Vx = random byte AND kk.
            0xC => self.i_cxkk(x, kk),
            // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
            0xD => self.i_dxyn(x, y, n),
            0xE => {
                match kk {
                    0x9E => self.i_ex9e(x),
                    0xA1 => self.i_exa1(x),
                    e    => panic!("Unknown LSB: {} in 0xE", e),
                }
            },
            // watch the least siginficant byte, in this case kk and match it;
            0xF => {
                match kk {
                    // Set Vx = delay timer value.
                    0x07 => self.i_fx07(x),
                    // Wait for a key press, store the value of the key in Vx.
                    0x0A => self.i_fx0a(x),
                    //  Set delay timer = Vx.
                    0x15 => self.i_fx15(x),
                    // Set sound timer = Vx.
                    0x18 => self.i_fx18(x),
                    // Set I = I + Vx.
                    0x1E => self.i_fx1e(x),
                    // Set I = location of sprite for digit Vx.
                    0x29 => self.i_fx29(x),
                    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
                    0x33 => self.i_fx33(x),
                    // Store registers V0 through Vx in memory starting at location I.
                    0x55 => self.i_fx55(x),
                    // Read registers V0 through Vx from memory starting at location I.
                    0x65 => self.i_fx65(x),
                    e => panic!("undefined LSB: 0x{:x} in 0xF pc: 0x{:x}", e, self.pc),
                }
            },
            e  => panic!("Unknown instruction: {}", e),
        };

        match programm_counter {
            InstructionOrd::Next => self.pc += 2,
            InstructionOrd::Skip => self.pc += 4,
            InstructionOrd::Jump(nnn) => {
                self.pc = nnn
            },
        }
    }

    fn i_00e0(&mut self) -> InstructionOrd {
        //println!("00E0: Clear the display.");

        self.vmem = [[0x0; SCR_HEIGHT]; SCR_WIDTH];
        self.vmem_changed = true;
        InstructionOrd::Next
    } 

    fn i_00ee(&mut self) -> InstructionOrd {
        let addr = self.pop();

        //println!("00EE: Return from subroutine to 0x{:x}", addr);

        InstructionOrd::Jump(addr)
    }

    fn i_0nnn(&mut self) -> InstructionOrd {
        //println!("0nnn ignored");

        InstructionOrd::Next
    }


    fn i_1nnn(&mut self, nnn: u16) -> InstructionOrd {
        //println!("Jump to location (0x{:x})", nnn);

        InstructionOrd::Jump(nnn)
    }

    fn i_2nnn(&mut self, nnn: u16) -> InstructionOrd {
        //println!("Call a subroutine at (0x{:x})", nnn);

        self.push();
        InstructionOrd::Jump(nnn)
    }

    fn i_3xkk(&mut self, x: usize, kk: u8) -> InstructionOrd {
        //println!("Skip next instruction if V[0x{:x}]: ({}) == kk:({})", x, self.v[x as usize], kk);

        if self.v[x] == kk {
            return InstructionOrd::Skip;
        }
        InstructionOrd::Next
    }
    
    fn i_4xkk(&mut self, x: usize, kk: u8) -> InstructionOrd {
        //println!("Skip next instruction if V[0x{:x}]: ({}) != kk:({})", x, self.v[x as usize], kk);

        if self.v[x] != kk {
            return InstructionOrd::Skip;
        }
        InstructionOrd::Next
    }

    fn i_5xy0(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Skip next instruction if V[0x{:x}]: ({}) != V[{}]: ({})",
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        if self.v[x] == self.v[y] {
            return InstructionOrd::Skip;
        }
        InstructionOrd::Next
    }

    fn i_6xkk(&mut self, x: usize, kk: u8) -> InstructionOrd {
        //println!("Set V[0x{:x}] = kk:({})", x, kk);

        self.v[x] = kk;
        InstructionOrd::Next
    }

    fn i_7xkk(&mut self, x: usize, kk: u8) -> InstructionOrd {
        //println!("Set V[0x{:x}] = V[0x{:x}]: ({}) + kk:({})",
        //    x,
        //    x, 
        //    self.v[x as usize],
        //    kk);

        let vx = self.v[x] as u16;
        let val = kk as u16;
        let res: u16 = vx + val;
        self.v[x] = res as u8;
        InstructionOrd::Next
    }

    fn i_8xy0(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}] = V[0x{:x}]: ({}).",
        //    x,
        //    y,
        //    self.v[y as usize]);

        self.v[x] = self.v[y]; 
        InstructionOrd::Next
    }

    fn i_8xy1(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}]: ({}) = V[0x{:x}]: ({}) OR V[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize],  
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        self.v[x] |= self.v[y];
        InstructionOrd::Next
    }

    fn i_8xy2(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}]: ({}) = V[0x{:x}]: ({}) AND V[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize],  
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        self.v[x] &= self.v[y];
        InstructionOrd::Next
    }

    fn i_8xy3(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}]: ({}) = V[0x{:x}]: ({}) XOR V[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize],  
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        self.v[x] ^= self.v[y];
        InstructionOrd::Next
    }

    fn i_8xy4(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}]: ({}) = V[0x{:x}]: ({}) + V[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize],
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        let res: u16 = vx + vy;
        self.v[x] = res as u8;
        
        if res > 255 { self.v[0xf] = 1 } else {  self.v[0xf] = 0 }
        InstructionOrd::Next
    }

    fn i_8xy5(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}]: ({}) = V[0x{:x}]: ({}) - V[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize],
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        if self.v[x] > self.v[y] { 
            self.v[0xf] = 1
        } else { 
            self.v[0xf] = 0
        }
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        InstructionOrd::Next
    }

    fn i_8xy6(&mut self, x: usize, _: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}]: ({}) = V[0x{:x}]: ({}) SHR 1",
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        self.v[0xf] = self.v[x] & 0x1;
        self.v[x] >>= 1;
        InstructionOrd::Next
    }

    fn i_8xy7(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Set Vx[0x{:x}]: ({}) = Vy[0x{:x}]: ({}) - Vx[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize],
        //    x,
        //    self.v[x as usize]);

        if self.v[y] > self.v[x] { 
            self.v[0xf] = 1
        } else { 
            self.v[0xf] = 0
        }
        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
        InstructionOrd::Next
    }

    fn i_8xye(&mut self, x: usize, _: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}]: ({}) = V[0x{:x}]: ({}) SHL 1",
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        self.v[0xf] = self.v[x] >> 7;
        self.v[x] <<= 1;
        InstructionOrd::Next
    }

    fn i_9xy0(&mut self, x: usize, y: usize) -> InstructionOrd {
        //println!("Skip next instruction if V[0x{:x}]: ({}) != V[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize],
        //    y,
        //    self.v[y as usize]);

        if self.v[x] != self.v[y] {
            return InstructionOrd::Skip;
        }
        InstructionOrd::Next
    }

    fn i_annn(&mut self, nnn: u16) -> InstructionOrd {
        //println!("Set I = 0x{:x}", nnn);

        self.i = nnn;
        InstructionOrd::Next
    }

    fn i_bnnn(&mut self, nnn: u16) -> InstructionOrd {
        //println!("Jump to location (0x{:x}) + V[0x0]: ({})",
        //    nnn,
        //    self.v[0]);

        InstructionOrd::Jump(nnn+self.v[0] as u16)
    }

    fn i_cxkk(&mut self, x: usize, kk: u8) -> InstructionOrd {
        println!("Generating random number in range 0.255");

        let mut rng = thread_rng();
        self.v[x] = rng.gen_range(0..255) & kk;
        InstructionOrd::Next
    }

    fn i_dxyn(&mut self, x: usize, y: usize, n: u8) -> InstructionOrd {
        //println!("Drawing at ({}, {}) {} bytes",
        //    self.v[x],
        //    self.v[y],
        //    n);

        /*
        let mut xi = self.v[x] as usize;
        let mut yj = self.v[y] as usize;
        self.v[0xf] = 0;
        for byte in &self.mem[self.i as usize..(self.i+n as u16) as usize] {
            let mut b = *byte;
            for _ in 0..8 {
                if xi > 63 { xi = 63 }
                if yj > 31 { yj = 31 }
                /*
                if (self.vmem[xi][yj] ^ b) == 1 {
                    self.v[0xf] = 1;
                } else {
                    self.v[0xf] = 0;
                }
                */
                self.vmem[xi][yj] ^= b >> 7;
                b <<= 1;
                xi += 1;
            }

            xi = self.v[x] as usize;
            yj += 1;
        }
        */

        self.v[0x0f] = 0;
        for byte in 0..n {
            let y = (self.v[y] + byte) % SCR_HEIGHT as u8;
            for bit in 0..8 {
                let x = (self.v[x] + bit) % SCR_WIDTH as u8;
                let color = (self.mem[(self.i + byte as u16) as usize] >> (7 - bit)) & 1;
                self.v[0x0f] |= color & self.vmem[x as usize][y as usize];
                self.vmem[x as usize][y as usize] ^= color;
            }
        }

        self.vmem_changed = true;
        InstructionOrd::Next
    }

    fn i_ex9e(&mut self, x: usize) -> InstructionOrd {
        //println!("Skip next instruction if key with the value of {} is pressed",
        //    self.v[x as usize]);

        if self.keys[self.v[x] as usize] {
            return InstructionOrd::Skip;
        }
        InstructionOrd::Next
    }

    fn i_exa1(&mut self, x: usize) -> InstructionOrd {
        //println!("Skip next instruction if key with the value of {} is not pressed",
        //    self.v[x as usize]);

        if !self.keys[self.v[x] as usize] {
            return InstructionOrd::Skip;
        }
        InstructionOrd::Next
    }

    fn i_fx07(&mut self, x: usize) -> InstructionOrd {
        //println!("Set V[0x{:x}] = delay timer value: ({})",
        //    x,
        //    self.dt);

        self.v[x] = self.dt;
        InstructionOrd::Next
    }

    fn i_fx0a(&mut self, x: usize) -> InstructionOrd {
        //println!("Store in V[{}]", x);
        
        self.key_waiting = true;
        self.key_to_store = Some(x);
        InstructionOrd::Next
    }

    fn i_fx15(&mut self, x: usize) -> InstructionOrd {
        //println!("Set delay timer = V[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize]);

        self.dt = self.v[x];
        InstructionOrd::Next
    }

    fn i_fx18(&mut self, x: usize) -> InstructionOrd {
        //println!("Set sound timer = V[0x{:x}]: ({})",
        //    x,
        //    self.v[x as usize]);

        self.st = self.v[x];
        InstructionOrd::Next
    }

    fn i_fx1e(&mut self, x: usize) -> InstructionOrd {
        //println!("Set I = I:(0x{:x}) + V[0x{:x}]: ({})",
        //    self.i,
        //    x,
        //    self.v[x as usize]);

        self.i += self.v[x] as u16;
        InstructionOrd::Next
    }

    fn i_fx29(&mut self, x: usize) -> InstructionOrd {
        //println!("I = address of memory location digit {}", self.v[x]);
        
        self.i = (self.v[x]*5) as u16;
        InstructionOrd::Next
    }

    fn i_fx33(&mut self, x: usize) -> InstructionOrd {
        //println!("hundreds {}, tens {}, ones {}", self.v[x] / 100, self.v[x] / 10 % 10, self.v[x] % 10);
       
        self.mem[self.i as usize] = self.v[x] / 100;
        self.mem[self.i as usize+1] = self.v[x] / 10 % 10;
        self.mem[self.i as usize+2] = self.v[x] % 10;
        InstructionOrd::Next
    }

    fn i_fx55(&mut self, x: usize) -> InstructionOrd {
        for i in 0..x + 1 {
            self.mem[self.i as usize+i] = self.v[i];
        }
        InstructionOrd::Next
    }

    fn i_fx65(&mut self, x: usize) -> InstructionOrd {
        for i in 0..x + 1 {
            self.v[i] = self.mem[self.i as usize+i];
        }
        InstructionOrd::Next
    }
}
