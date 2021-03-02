use rand::Rng;
use std::error::Error;
use std::fs::File;
use std::io::Read;

// Constant definitions
const FONT_SET: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
const FONT_SET_SIZE: usize = FONT_SET.len();
const MEM_SIZE: usize = 4096;
const STACK_SIZE: usize = 24;
const REG_SIZE: usize = 16;
const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const KEY_SIZE: usize = 16;
const PC_START: usize = 0x200;

struct Chip8 {
    // Current opcode
    opcode: u16,

    memory: [u8; MEM_SIZE],

    // CPU Register v0 - vF, vF is used as flag
    v: [u8; REG_SIZE],

    // Address Register
    addr_reg: u16,

    // Program Counter starts at 0x200
    pc: usize,

    // Only used to store Return Address (16 bit)
    stack: [u16; STACK_SIZE],

    // Stack pointer
    sp: u8,

    // Screen 64 x 32 pixels, monochrome
    screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],

    // Array to store current of hex keyboard (0x0 - 0xF)
    key: [u8; KEY_SIZE],

    // Timer at 60 Hz, count down to 0 from current value
    delay_timer: u8,

    // Timer at 60 Hz, count down to 0 from current value.
    // Beeping sound is made when it reaches 0.
    sound_timer: u8,
}

impl Chip8 {
    fn init() -> Chip8 {
        let mut emu = Chip8 {
            // Initialize registers and memory
            pc: PC_START,
            opcode: 0,
            addr_reg: 0,
            sp: 0,

            // Clear registers v0 - vF
            v: [0; REG_SIZE],
            // Clear stack
            stack: [0; STACK_SIZE],
            // Clear memory
            memory: [0; MEM_SIZE],
            // Clear screen
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT],

            // Initialize timers
            delay_timer: 0,
            sound_timer: 0,

            // Initialize input
            key: [0; KEY_SIZE],
        };

        // Load fontset
        for i in 0..FONT_SET_SIZE {
            emu.memory[i] = FONT_SET[i];
        }

        emu
    }

    fn load_game(&mut self, filename: String) -> Result<usize, Box<dyn Error>> {
        let mut f = File::open(filename)?;
        let mut buffer = Vec::<u8>::new();
        let buf_size = f.read_to_end(&mut buffer)?;

        for i in 0..buf_size - 1 {
            // excluding EOF
            self.memory[PC_START + i] = buffer[i];
        }

        Ok(buf_size - 1) // excluding EOF
    }

    fn emulate(&mut self) {
        // Fetch opcode
        self.opcode = memory[pc] << 8 | memory[pc + 1];

        // TODO: Can we just have opcode as local variable only in this function?
        // Decode & execute opcode
        match (self.opcode & 0xF000) {
            0x0000 => opcode_0(),
            0x1000 => opcode_1(), 
            0x2000 => opcode_2(),
            0x3000 => opcode_3(),
            0x4000 => opcode_4(),
            0x5000 => opcode_5(),
            0x6000 => opcode_6(),
            0x7000 => opcode_7(),
            0x8000 => opcode_8(),
            0x9000 => opcode_9(),
            0xA000 => opcode_A(),
            0xB000 => opcode_B(),
            0xC000 => opcode_C(),
            0xD000 => opcode_D(),
            0xE000 => opcode_E(),
            0xF000 => opcode_F(),
        };
        // Update timers
    }

    fn opcode_0(&mut self) {
        match self.opcode {
            0x00E0 => {
                // Clears the screen
                self.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
                self.pc += 2;
            },
            0x00EE => {
                // Returns from a subroutine
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            },
            _ => {}      // TODO: calls machine code routine at 0x0NNN
        };
    }

    fn opcode_1(&mut self) {
        // Opcode: 1NNN
        // Goto 0x0NNN
        self.pc = self.opcode & 0x0FFF;
    }

    fn opcode_2(&mut self) {
        // Opcode: 2NNN
        // Calls subroutine at 0x0NNN
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFF;
    }

    fn opcode_3(&mut self) {
        // Opcode: 3XNN
        // Skips the next instruction if vX == NN
        let x = (self.opcode & 0x0F00) >> 8;
        let nn = self.opcode & 0x00FF;

        if self.v[x] == nn {
            self.pc += 4;
        };
    }

    fn opcode_4(&mut self) {
        // Opcode: 4XNN
        // Skips the next instruction if vX != NN
        let x = (self.opcode & 0x0F00) >> 8;
        let nn = self.opcode & 0x00FF;

        if self.v[x] != nn {
            self.pc += 4;
        };
    }

    fn opcode_5(&mut self) {
        // Opcode: 5XY0, let's just ignore if 4 lsb is not 0
        // Skips the next instruction if vX == vY
        let x = (self.opcode & 0x0F00) >> 8;
        let y = (self.opcode & 0x00F0) >> 4;

        if self.v[x] == self.v[y] {
            self.pc += 4;
        };
    }

    fn opcode_6(&mut self) {
        // Opcode: 6XNN
        // Sets vX to NN
        self.v[(self.opcode & 0x0F00) >> 8] = self.opcode & 0x00FF;
        self.pc += 2;
    }

    fn opcode_7(&mut self) {
        // Opcode: 7XNN
        // vX += NN
        self.v[(self.opcode & 0x0F00) >> 8] += self.opcode & 0x00FF;
        self.pc += 2;
    }

    fn opcode_8(&mut self) {
        // Opcode: 8XY_
        let x = (self.opcode & 0x0F00) >> 8;
        let y = (self.opcode & 0x00F0) >> 4;

        match (self.opcode & 0x000F) {
            0x0000 => self.v[x] = self.v[y],  // vX = vY
            0x0001 => self.v[x] |= self.v[y], // vX = vX | vY
            0x0002 => self.v[x] &= self.v[y], // vX = vX & vY
            0x0003 => self.v[x] ^= self.v[y], // vX = vX XOR vY
            0x0004 => {
                // vX = vX + vY
                // set vF to 1 when there's a carry, set to 0, otherwise
                if self.v[y] > (0xFF - self.v[x]) {
                    self.v[0x0F] = 1;
                } else {
                    self.v[0x0F] = 0;
                };

                self.v[x] += self.v[y];
                self.pc += 2;
            }
            0x0005 => {
                // vX = vX - vY
                // set vF to 0 when there's a borrow, set to 1, otherwise
                if self.v[x] < self.v[y] {
                    self.v[0x0F] = 0;
                } else {
                    self.v[0x0F] = 1;
                };

                self.v[x] -= self.v[y];
                self.pc += 2;
            }
            0x0006 => {
                // Store lsb of vX in vF
                // vX >>= 1
                // TODO: store lsb of vX to vF
                self.v[x] >>= 1;
            }
            0x0007 => {
                // vX = vY - vX
                // sets vF to 0 there's a borrow, set to 1, otherwise
                if self.v[y] < self.v[x] {
                    self.v[0x0F] = 0;
                } else {
                    self.v[0x0F] = 1;
                };

                self.v[x] = self.v[y] - self.v[x];
                self.pc += 2;
            }
            0x000E => {
                // Store msb of vX in vF
                // vX <<= 1
                // TODO: store msb of vX to vF
                self.v[x] <<= 1;
            }
            _ => {} // TODO: Unknown opcode
        };
    }

    fn opcode_9(&mut self) {
        // Opcode: 9XY0, let's just ignore if 4 lsb is not 0
        // Skips the next instruction if vX != vY
        let x = (self.opcode & 0x0F00) >> 8;
        let y = (self.opcode & 0x00F0) >> 4;

        if self.v[x] != self.v[y] {
            self.pc += 4;
        };
    }

    fn opcode_A(&mut self) {
        // Opcode: ANNN
        // Sets addr_reg (I) to NNN
        self.addr_reg = self.opcode & 0x0FFF;
        self.pc += 2;
    }

    fn opcode_B(&mut self) {
        // Opcode: BNNN
        // Go to address v0 + NNN
        self.pc = self.v[0] + (self.opcode & 0x0FFF);
    }

    fn opcode_C(&mut self) {
        // Opcode: CXNN
        // vX = rand(0 to 255) & NN
        let secret = rand::thread_rng().gen_range(0, 256);
        let x = (self.opcode & 0x0F00) >> 8;
        let nn = self.opcode & 0x00FF;

        self.v[x] = secret & nn;
        self.pc += 2;
    }

    fn opcode_D(&self) {
        // Opcode: DXYN
        // Draw a sprite at coordinate (vX, vY), w:8px h:(N+1)px
        // Each row of 8 pixels is read as bit-coded from memory at addr_reg
        // addr_reg doesn't change after this instruction
        // Sets vF to 1 if any screen pixels are flipped from set to unset, set to 0, otherwise
        // TODO: draw function
    }

    fn opcode_E(&mut self) {
        match (self.opcode & 0x00FF) {
            0x009E => {
                // Opcode: EX9E
                // Skips the next instruction if key[vX] != 0 (it's pressed)
                let x = (self.opcode & 0x0F00) >> 8;

                if self.key[self.v[x]] != 0 {
                    self.pc += 4;
                };
            }
            0x00A1 => {
                // Opcode: EXA1
                // Skips the next instruction if key[vX] == 0 (it's not pressed)
                let x = (self.opcode & 0x0F00) >> 8;

                if self.key[self.v[x]] == 0 {
                    self.pc += 4;
                };
            }
            _ => (), // TODO: unknown code
        };
    }

    fn opcode_F(&mut self) {
        // Opcode: FX__
        let x = (self.opcode & 0x0F00) >> 8;

        match (self.opcode & 0x00FF) {
            0x0007 => self.v[x] = self.delay_timer,
            0x000A => {} // TODO: await key
            0x0015 => self.delay_timer = self.v[x],
            0x0018 => self.sound_timer = self.v[x],
            0x001E => self.addr_reg += self.v[x],
            0x0029 => {} // TODO: still dont understand
            0x0033 => {
                // Takes BCD form of vX
                // Stores the hundreds digit at memory[addr_reg]
                // Stores the tens digit at memory[addr_reg + 1]
                // Stores the ones digit at memory[addr_reg + 2]
                self.memory[addr_reg] = self.v[x] / 100;
                self.memory[addr_reg + 1] = (self.v[x] / 10) % 10;
                self.memory[addr_reg + 2] = (self.v[x] % 100) % 10;

                self.pc += 2;
            },
            0x0055 => {} // TODO: reg_dump
            0x0065 => {} // TODO: reg_load
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let emu = Chip8::init();

        assert_eq!(emu.pc, PC_START);
        assert_eq!(emu.opcode, 0);
        assert_eq!(emu.addr_reg, 0);
        assert_eq!(emu.sp, 0);
        assert_eq!(emu.v, [0 as u8; REG_SIZE]); // register is 8 bit wide
        assert_eq!(emu.stack, [0 as u16; STACK_SIZE]); // stack is 16 bit wide
        assert_eq!(emu.memory[0..FONT_SET_SIZE], FONT_SET);
        assert_eq!(
            emu.memory[FONT_SET_SIZE..MEM_SIZE],
            [0 as u8; MEM_SIZE - FONT_SET_SIZE]
        ); // memory is 8 bit wide
        assert_eq!(emu.screen, [0 as u8; SCREEN_WIDTH * SCREEN_HEIGHT]); // screen use u8
        assert_eq!(emu.delay_timer, 0);
        assert_eq!(emu.sound_timer, 0);
        assert_eq!(emu.key, [0; KEY_SIZE]); // key use u8
    }

    #[test]
    fn test_load_game() {
        let mut emu = Chip8::init();
        let size = emu.load_game(String::from("dummy_game")).unwrap();

        assert_eq!(emu.memory[PC_START..PC_START + size], [49, 50, 51, 52]);
    }
}
