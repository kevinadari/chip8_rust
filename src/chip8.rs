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
const MEM_SIZE: usize = 4096;
const STACK_SIZE: usize = 24;
const REG_SIZE: usize = 16;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const KEY_SIZE: usize = 16;
const PC_START: usize = 0x200;

pub struct Chip8 {
    // Current opcode
    pub opcode: usize,

    memory: [u8; MEM_SIZE],

    // CPU Register v0 - vF, vF is used as flag
    v: [u8; REG_SIZE],

    // Address Register
    addr_reg: usize,

    // Program Counter starts at 0x200
    pc: usize,

    // Only used to store Return Address (16 bit)
    stack: [usize; STACK_SIZE],

    // Stack pointer
    sp: usize,

    // Screen 64 x 32 pixels, monochrome
    pub screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],

    // Flag for drawing screen
    pub draw_flag: bool,

    // Array to store current of hex keyboard (0x0 - 0xF)
    key: [u8; KEY_SIZE],

    // Timer at 60 Hz, count down to 0 from current value
    delay_timer: u8,

    // Timer at 60 Hz, count down to 0 from current value.
    // Beeping sound is made when it reaches 0.
    sound_timer: u8,
}

impl Chip8 {
    pub fn init() -> Chip8 {
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
            draw_flag: false,

            // Initialize timers
            delay_timer: 0,
            sound_timer: 0,

            // Initialize input
            key: [0; KEY_SIZE],
        };

        // Load fontset
        for (i, byte) in FONT_SET.iter().enumerate() {
            emu.memory[i] = *byte;
        }

        emu
    }

    pub fn load_game(&mut self, filename: &str) -> Result<usize, Box<dyn Error>> {
        let mut f = File::open(filename)?;
        let mut buffer = Vec::<u8>::new();
        let buf_size = f.read_to_end(&mut buffer)?;

        // excluding EOF
        self.memory[PC_START..(buf_size + PC_START)]
            .clone_from_slice(&buffer[..buf_size]);

        Ok(buf_size) // excluding EOF
    }

    pub fn emulate(&mut self) {
        // Fetch opcode
        self.opcode = (self.memory[self.pc] as usize) << 8 | self.memory[self.pc + 1] as usize;

        // Decode & execute opcode
        match self.opcode & 0xF000 {
            0x0000 => self.opcode_0(),
            0x1000 => self.opcode_1(),
            0x2000 => self.opcode_2(),
            0x3000 => self.opcode_3(),
            0x4000 => self.opcode_4(),
            0x5000 => self.opcode_5(),
            0x6000 => self.opcode_6(),
            0x7000 => self.opcode_7(),
            0x8000 => self.opcode_8(),
            0x9000 => self.opcode_9(),
            0xA000 => self.opcode_a(),
            0xB000 => self.opcode_b(),
            0xC000 => self.opcode_c(),
            0xD000 => self.opcode_d(),
            0xE000 => self.opcode_e(),
            0xF000 => self.opcode_f(),
            _ => panic!("Unknown opcode: 0x{:04X}!", self.opcode), // Unreachable
        };

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        };

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // TODO: sound beep
            };
            self.sound_timer -= 1;
        };
    }

    fn opcode_0(&mut self) {
        match self.opcode {
            0x00E0 => {
                // Clears the screen
                self.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
                self.pc += 2;
                self.draw_flag = true;
            }
            0x00EE => {
                // Returns from a subroutine
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            }
            _ => panic!("Unknown opcode: 0x{:04X}!", self.opcode),
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
        self.stack[self.sp] = self.pc + 2;
        self.sp += 1;
        self.pc = self.opcode & 0x0FFF;
    }

    fn opcode_3(&mut self) {
        // Opcode: 3XNN
        // Skips the next instruction if vX == NN
        let x = (self.opcode & 0x0F00) >> 8;
        let nn = (self.opcode & 0x00FF) as u8;

        if self.v[x] == nn {
            self.pc += 4;
        } else {
            self.pc += 2;
        };
    }

    fn opcode_4(&mut self) {
        // Opcode: 4XNN
        // Skips the next instruction if vX != NN
        let x = (self.opcode & 0x0F00) >> 8;
        let nn = (self.opcode & 0x00FF) as u8;

        if self.v[x] != nn {
            self.pc += 4;
        } else {
            self.pc += 2;
        };
    }

    fn opcode_5(&mut self) {
        // Opcode: 5XY0, let's just ignore if 4 lsb is not 0
        // Skips the next instruction if vX == vY
        let x = (self.opcode & 0x0F00) >> 8;
        let y = (self.opcode & 0x00F0) >> 4;

        if self.v[x] == self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        };
    }

    fn opcode_6(&mut self) {
        // Opcode: 6XNN
        // Sets vX to NN
        self.v[(self.opcode & 0x0F00) >> 8] = (self.opcode & 0x00FF) as u8;
        self.pc += 2;
    }

    fn opcode_7(&mut self) {
        // Opcode: 7XNN
        // vX += NN
        let index: usize = (self.opcode & 0x0F00) >> 8;
        self.v[index] = self.v[index].wrapping_add((self.opcode & 0x00FF) as u8);
        self.pc += 2;
    }

    fn opcode_8(&mut self) {
        // Opcode: 8XY_
        let x = (self.opcode & 0x0F00) >> 8;
        let y = (self.opcode & 0x00F0) >> 4;

        match self.opcode & 0x000F {
            0x0000 => {
                self.v[x] = self.v[y];
                self.pc += 2;
            }
            0x0001 => {
                self.v[x] |= self.v[y];
                self.pc += 2;
            }
            0x0002 => {
                self.v[x] &= self.v[y];
                self.pc += 2;
            }
            0x0003 => {
                self.v[x] ^= self.v[y];
                self.pc += 2;
            }
            0x0004 => {
                // vX = vX + vY
                // set vF to 1 when there's a carry, set to 0, otherwise
                let (result, carry) = self.v[x].overflowing_add(self.v[y]);

                if carry {
                    self.v[0x0F] = 1;
                } else {
                    self.v[0x0F] = 0;
                };

                self.v[x] = result;
                self.pc += 2;
            }
            0x0005 => {
                // vX = vX - vY
                // set vF to 0 when there's a borrow, set to 1, otherwise
                let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);

                if borrow {
                    self.v[0x0F] = 0;
                } else {
                    self.v[0x0F] = 1;
                };

                self.v[x] = result;
                self.pc += 2;
            }
            0x0006 => {
                // Store lsb of vX in vF
                // vX >>= 1
                self.v[0x0F] = self.v[x] & 0x01;
                self.v[x] >>= 1;
                self.pc += 2;
            }
            0x0007 => {
                // vX = vY - vX
                // sets vF to 0 there's a borrow, set to 1, otherwise
                let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);

                if borrow {
                    self.v[0x0F] = 0;
                } else {
                    self.v[0x0F] = 1;
                };

                self.v[x] = result;
                self.pc += 2;
            }
            0x000E => {
                // Store msb of vX in vF
                // vX <<= 1
                self.v[0x0F] = (self.v[x] & 0x80) >> 7;
                self.v[x] <<= 1;
                self.pc += 2;
            }
            _ => panic!("Unknown opcode: 0x{:04X}!", self.opcode),
        };
    }

    fn opcode_9(&mut self) {
        // Opcode: 9XY0, let's just ignore if 4 lsb is not 0
        // Skips the next instruction if vX != vY
        let x = (self.opcode & 0x0F00) >> 8;
        let y = (self.opcode & 0x00F0) >> 4;

        if self.v[x] != self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        };
    }

    fn opcode_a(&mut self) {
        // Opcode: ANNN
        // Sets addr_reg (I) to NNN
        self.addr_reg = self.opcode & 0x0FFF;
        self.pc += 2;
    }

    fn opcode_b(&mut self) {
        // Opcode: BNNN
        // Go to address v0 + NNN
        self.pc = self.v[0] as usize + (self.opcode & 0x0FFF);
    }

    fn opcode_c(&mut self) {
        // Opcode: CXNN
        // vX = rand(0 to 255) & NN
        let secret = rand::thread_rng().gen_range(0, 256) as u8;
        let x = (self.opcode & 0x0F00) >> 8;
        let nn = (self.opcode & 0x00FF) as u8;

        self.v[x] = secret & nn;
        self.pc += 2;
    }

    fn opcode_d(&mut self) {
        // Opcode: DXYN
        // Draw a sprite at coordinate (vX, vY), w:8px h:Npx
        // Each row of 8 pixels is read as bit-coded from memory at addr_reg
        // addr_reg doesn't change after this instruction
        // Sets vF to 1 if any screen pixels are flipped from set to unset, set to 0, otherwise
        let x = self.v[(self.opcode & 0x0F00) >> 8] as usize;
        let y = self.v[(self.opcode & 0x00F0) >> 4] as usize;
        let h = self.opcode & 0x000F; // Do not add 1 because for loop start from 0
        let mut sprite;

        self.v[0x0F] = 0;

        for y_row in 0..h {
            sprite = self.memory[y_row + self.addr_reg];
            // println!("sprite[{}]: {:02X}", y_row, sprite); // Debug
            for x_col in 0..8 {
                if sprite & (0x80 >> x_col) != 0 {
                    let coordinate = x + x_col + ((y + y_row) * SCREEN_WIDTH);

                    // Sets vF if pixel is flipped from 1 to 0
                    if self.screen[coordinate] == 1 {
                        self.v[0x0F] = 1;
                    }
                    self.screen[coordinate] ^= 1;
                }
            }
        }

        self.pc += 2;
        self.draw_flag = true;
    }

    fn opcode_e(&mut self) {
        let x = (self.opcode & 0x0F00) >> 8;
        match self.opcode & 0x00FF {
            0x009E => {
                // Opcode: EX9E
                // Skips the next instruction if key[vX] != 0 (it's pressed)
                if self.key[self.v[x] as usize] != 0 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            0x00A1 => {
                // Opcode: EXA1
                // Skips the next instruction if key[vX] == 0 (it's not pressed)
                if self.key[self.v[x] as usize] == 0 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                };
            }
            _ => panic!("Unknown opcode: 0x{:04X}!", self.opcode),
        };
    }

    fn opcode_f(&mut self) {
        // Opcode: FX__
        let x = (self.opcode & 0x0F00) >> 8;

        match self.opcode & 0x00FF {
            0x0007 => {
                self.v[x] = self.delay_timer;
                self.pc += 2;
            }
            0x000A => {
                // TODO: wait for key pressed and store it in vX. How?
            }
            0x0015 => {
                self.delay_timer = self.v[x];
                self.pc += 2;
            }
            0x0018 => {
                self.sound_timer = self.v[x];
                self.pc += 2;
            }
            0x001E => {
                self.addr_reg += self.v[x] as usize;
                self.pc += 2;
            }
            0x0029 => {
                // Assign address of font set of character in vX to addr_reg
                // At this point vX shall have values in [0 to F]
                if self.v[x] > 0x0F {
                    panic!("Font set is only for character 0 to F!");
                };

                self.addr_reg = (self.v[x] * 5) as usize;
                self.pc += 2;
            }
            0x0033 => {
                // Takes BCD form of vX
                // Stores the hundreds digit at memory[addr_reg]
                // Stores the tens digit at memory[addr_reg + 1]
                // Stores the ones digit at memory[addr_reg + 2]
                self.memory[self.addr_reg] = self.v[x] / 100;
                self.memory[self.addr_reg + 1] = (self.v[x] / 10) % 10;
                self.memory[self.addr_reg + 2] = (self.v[x] % 100) % 10;

                self.pc += 2;
            }
            0x0055 => {
                // Stores v[0 to X] in memory starting at addr_reg
                // addr_reg is not modified
                for i in 0..x + 1 {
                    self.memory[self.addr_reg + i] = self.v[i];
                }
                self.pc += 2;
            }
            0x0065 => {
                // Fills v[0 to X] by value in memory starting addr_reg
                // addr_reg is not modified
                for i in 0..x + 1 {
                    self.v[i] = self.memory[self.addr_reg + i];
                }
                self.pc += 2;
            }
            _ => panic!("Unknown opcode: 0x{:04X}!", self.opcode),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_opcode(emu: &mut Chip8, opcode: &[u16]) {
        let mut pc = PC_START;

        for op in opcode {
            emu.memory[pc] = (op >> 8) as u8;
            emu.memory[pc + 1] = (op & 0x00FF) as u8;
            pc += 2;
        }
    }

    #[test]
    fn test_init() {
        let emu = Chip8::init();

        assert_eq!(emu.pc, PC_START);
        assert_eq!(emu.opcode, 0);
        assert_eq!(emu.addr_reg, 0);
        assert_eq!(emu.sp, 0);
        assert_eq!(emu.v, [0_u8; REG_SIZE]); // register is 8 bit wide
        assert_eq!(emu.stack, [0; STACK_SIZE]); // stack is 16 bit wide
        assert_eq!(emu.memory[0..FONT_SET.len()], FONT_SET);
        assert_eq!(
            emu.memory[FONT_SET.len()..MEM_SIZE],
            [0_u8; MEM_SIZE - FONT_SET.len()]
        ); // memory is 8 bit wide
        assert_eq!(emu.screen, [0_u8; SCREEN_WIDTH * SCREEN_HEIGHT]); // screen use u8
        assert_eq!(emu.draw_flag, false);
        assert_eq!(emu.delay_timer, 0);
        assert_eq!(emu.sound_timer, 0);
        assert_eq!(emu.key, [0; KEY_SIZE]); // key use u8
    }

    #[test]
    fn test_load_game() {
        let mut emu = Chip8::init();
        let size = emu.load_game("IBM Logo.ch8").unwrap();

        assert_eq!(size, 132);
    }

    #[test]
    fn test_opcode_0_clear_screen() {
        let mut emu = Chip8::init();

        // Init screen
        emu.screen = [1; SCREEN_WIDTH * SCREEN_HEIGHT]; // Sets all pixels
        assert_eq!(emu.screen, [1; SCREEN_WIDTH * SCREEN_HEIGHT]); // Confirms all pixels are set

        // Opcode 00E0: Clear screen
        store_opcode(&mut emu, &[0x00E0]);

        // Emulate
        emu.emulate();
        assert_eq!(emu.screen, [0; SCREEN_WIDTH * SCREEN_HEIGHT]);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_0_return() {
        let mut emu = Chip8::init();

        // Init stack
        emu.stack[emu.sp] = 0x0251;
        emu.sp += 1;

        // Opcode 00EE: Returns from subroutine
        store_opcode(&mut emu, &[0x00EE]);

        // Emulate
        emu.emulate();
        assert_eq!(emu.sp, 0);
        assert_eq!(emu.pc, 0x0251);
    }

    #[test]
    #[should_panic(expected = "Unknown opcode: 0x00F1!")]
    fn test_opcode_0_unknown() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0x00F1]);

        emu.emulate();
    }

    #[test]
    fn test_opcode_1() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0x1224]);

        emu.emulate();

        assert_eq!(emu.pc, 0x0224);
    }

    #[test]
    fn test_opcode_2() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0x2301]);

        emu.emulate();

        assert_eq!(emu.pc, 0x0301);
        assert_eq!(emu.stack[emu.sp - 1], PC_START + 2);
    }

    #[test]
    fn test_opcode_3_skip() {
        let mut emu = Chip8::init();

        // Init
        emu.v[0] = 0x23;

        store_opcode(&mut emu, &[0x3023]);

        emu.emulate();

        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    fn test_opcode_3_no_skip() {
        let mut emu = Chip8::init();

        // Init
        emu.v[0] = 0x23;

        store_opcode(&mut emu, &[0x3021]);

        emu.emulate();

        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_4_skip() {
        let mut emu = Chip8::init();

        // Init
        emu.v[3] = 0x23;

        store_opcode(&mut emu, &[0x4321]);

        emu.emulate();

        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    fn test_opcode_4_no_skip() {
        let mut emu = Chip8::init();

        // Init
        emu.v[3] = 0x23;

        store_opcode(&mut emu, &[0x4323]);

        emu.emulate();

        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_5_skip() {
        let mut emu = Chip8::init();

        // Init
        emu.v[2] = 0x23;
        emu.v[3] = 0x23;

        store_opcode(&mut emu, &[0x5230]);

        emu.emulate();

        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    fn test_opcode_5_no_skip() {
        let mut emu = Chip8::init();

        // Init
        emu.v[2] = 0x23;
        emu.v[3] = 0x24;

        store_opcode(&mut emu, &[0x5230]);

        emu.emulate();

        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_6() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0x6AFF]);

        emu.emulate();

        assert_eq!(emu.v[0xA], 0x00FF);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_7() {
        let mut emu = Chip8::init();

        // Init
        emu.v[4] = 4;

        store_opcode(&mut emu, &[0x7422]);

        emu.emulate();

        assert_eq!(emu.v[4], 0x0026);
        assert_eq!(emu.v[0xF], 0); // Check carry flag
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_7_overflow() {
        let mut emu = Chip8::init();

        // Init
        emu.v[4] = 1;

        store_opcode(&mut emu, &[0x74FF]);

        emu.emulate();

        assert_eq!(emu.v[4], 0);
        assert_eq!(emu.v[0xF], 0); // Check carry flag
        assert_eq!(emu.pc, PC_START + 2);
    }
    #[test]
    fn test_opcode_8_0() {
        let mut emu = Chip8::init();

        // Init
        emu.v[6] = 0x12;

        store_opcode(&mut emu, &[0x8260]);

        emu.emulate();

        assert_eq!(emu.v[2], 0x12);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_8_1() {
        // vX = vX | vY
        let mut emu = Chip8::init();

        // Init
        emu.v[2] = 0xAB;
        emu.v[7] = 0x14;

        store_opcode(&mut emu, &[0x8271]);

        emu.emulate();

        assert_eq!(emu.v[2], 0xBF);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_8_2() {
        // vX = vX & vY
        let mut emu = Chip8::init();

        // Init
        emu.v[2] = 0xAB;
        emu.v[7] = 0x14;

        store_opcode(&mut emu, &[0x8272]);

        emu.emulate();

        assert_eq!(emu.v[2], 0);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_8_3() {
        // vX = vX ^ vY
        let mut emu = Chip8::init();

        // Init
        emu.v[2] = 0xAB;
        emu.v[7] = 0x14;

        store_opcode(&mut emu, &[0x8273]);

        emu.emulate();

        assert_eq!(emu.v[2], 0xBF);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_8_4() {
        // vX += vY
        let mut emu = Chip8::init();

        // Init
        emu.v[1] = 0xFF;
        emu.v[2] = 0xFF;

        emu.v[4] = 0x12;
        emu.v[9] = 0x13;

        store_opcode(&mut emu, &[0x8124, 0x8494]);

        // Carry
        emu.emulate();
        assert_eq!(emu.v[1], 0xFE);
        assert_eq!(emu.v[0xF], 1);
        assert_eq!(emu.pc, PC_START + 2);

        // No carry
        emu.emulate();
        assert_eq!(emu.v[4], 0x25);
        assert_eq!(emu.v[0xF], 0);
        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    fn test_opcode_8_5() {
        // vX -= vY
        let mut emu = Chip8::init();

        // Init
        emu.v[0x5] = 0x78;
        emu.v[0xA] = 0x24;

        emu.v[1] = 0x0;
        emu.v[2] = 0x1;

        store_opcode(&mut emu, &[0x85A5, 0x8125]);

        // No borrow
        emu.emulate();
        assert_eq!(emu.v[5], 0x54);
        assert_eq!(emu.v[0xF], 1);
        assert_eq!(emu.pc, PC_START + 2);

        // Borrow
        emu.emulate();
        assert_eq!(emu.v[1], 0xFF);
        assert_eq!(emu.v[0xF], 0);
        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    fn test_opcode_8_6() {
        // vX >>= 1, lsb stored in vF
        let mut emu = Chip8::init();

        // Init
        emu.v[2] = 0xFD;

        store_opcode(&mut emu, &[0x8236, 0x8236]);

        // lsb is 1
        emu.emulate();
        assert_eq!(emu.v[2], 0x7E);
        assert_eq!(emu.v[0xF], 1);
        assert_eq!(emu.pc, PC_START + 2);

        // lsb is 0
        emu.emulate();
        assert_eq!(emu.v[2], 0x3F);
        assert_eq!(emu.v[0xF], 0);
        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    fn test_opcode_8_7() {
        // vX = vY - vX
        let mut emu = Chip8::init();

        // Init
        emu.v[1] = 0x25;
        emu.v[0] = 0x11;

        emu.v[4] = 0x03;
        emu.v[3] = 0x04;

        store_opcode(&mut emu, &[0x8017, 0x8347]);

        // No borrow
        emu.emulate();
        assert_eq!(emu.v[0], 0x14);
        assert_eq!(emu.v[0x0F], 1);
        assert_eq!(emu.pc, PC_START + 2);

        // Borrow
        emu.emulate();
        assert_eq!(emu.v[3], 0xFF);
        assert_eq!(emu.v[0x0F], 0);
        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    fn test_opcode_8_e() {
        // vX <<= 1, msb stored in vF
        let mut emu = Chip8::init();

        // Init
        emu.v[2] = 0xBD;

        store_opcode(&mut emu, &[0x823E, 0x823E]);

        // msb is 1
        emu.emulate();
        assert_eq!(emu.v[2], 0x7A);
        assert_eq!(emu.v[0xF], 1);
        assert_eq!(emu.pc, PC_START + 2);

        // msb is 0
        emu.emulate();
        assert_eq!(emu.v[2], 0xF4);
        assert_eq!(emu.v[0xF], 0);
        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    #[should_panic(expected = "Unknown opcode: 0x823A!")]
    fn test_opcode_8_unknown() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0x823A]);
        emu.emulate();
    }

    #[test]
    fn test_opcode_9_skip() {
        let mut emu = Chip8::init();

        // Init
        emu.v[1] = 2;
        emu.v[2] = 3;

        store_opcode(&mut emu, &[0x9120]);

        emu.emulate();

        assert_eq!(emu.pc, PC_START + 4);
    }

    #[test]
    fn test_opcode_9_no_skip() {
        let mut emu = Chip8::init();

        // Init
        emu.v[1] = 2;
        emu.v[2] = 2;

        store_opcode(&mut emu, &[0x9120]);

        emu.emulate();

        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_a() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0xA4FE]);

        emu.emulate();
        assert_eq!(emu.addr_reg, 0x4FE);
    }

    #[test]
    fn test_opcode_b() {
        let mut emu = Chip8::init();

        // Init
        emu.v[0] = 0xFF;

        store_opcode(&mut emu, &[0xBFFF]);

        emu.emulate();
        assert_eq!(emu.pc, 0xFF + 0xFFF);
    }

    #[test]
    fn test_opcode_c_nn_00() {
        let mut emu = Chip8::init();

        // Init
        emu.v[0] = 0xFF;

        store_opcode(&mut emu, &[0xC000]);

        emu.emulate();
        assert_eq!(emu.v[0], 0);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_c_nn_ff() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0xC07F]);

        emu.emulate();
        assert_eq!((emu.v[0] & 0x80) >> 7, 0);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    #[should_panic(expected = "Unknown opcode: 0xE2FF")]
    fn test_opcode_e_unknown() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0xE2FF]);

        emu.emulate();
    }

    #[test]
    fn test_opcode_f_07() {
        // vX = delay_timer
        let mut emu = Chip8::init();

        // Init
        emu.delay_timer = 10;

        store_opcode(&mut emu, &[0xF207]);

        emu.emulate();
        assert_eq!(emu.v[2], 10);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_f_15() {
        // delay_timer = vX
        let mut emu = Chip8::init();

        // Init
        emu.v[5] = 19;

        store_opcode(&mut emu, &[0xF515]);

        emu.emulate();
        assert_eq!(emu.delay_timer, 18);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_f_18() {
        // sound_timer = vX
        let mut emu = Chip8::init();

        // Init
        emu.v[5] = 45;

        store_opcode(&mut emu, &[0xF518]);

        emu.emulate();
        assert_eq!(emu.sound_timer, 44);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_f_1e() {
        // addr_reg += vX, vF is not affected
        let mut emu = Chip8::init();

        // Init
        emu.addr_reg = 4;
        emu.v[5] = 19;

        store_opcode(&mut emu, &[0xF51E]);

        emu.emulate();
        assert_eq!(emu.addr_reg, 23);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    #[should_panic(expected = "Font set is only for character 0 to F!")]
    fn test_opcode_f_29_unknown_font() {
        let mut emu = Chip8::init();

        // Init
        emu.v[5] = 19;

        store_opcode(&mut emu, &[0xF529]);

        emu.emulate();
    }

    #[test]
    fn test_opcode_f_29() {
        let mut emu = Chip8::init();
        let mut op = [0; 16];

        // Init
        for (i, item) in op.iter_mut().enumerate() {
            emu.v[i] = i as u8;
            *item = (0xF029 | (i << 8)) as u16;
        }

        store_opcode(&mut emu, &op);

        for i in 0..0x10 {
            emu.emulate();
            assert_eq!(emu.addr_reg, (i * 5));
            assert_eq!(emu.pc, PC_START + (i + 1) * 2);
        }
    }

    #[test]
    fn test_opcode_f_33() {
        let mut emu = Chip8::init();

        // Init
        emu.addr_reg = 0x504;
        emu.v[5] = 237;

        store_opcode(&mut emu, &[0xF533]);

        emu.emulate();
        assert_eq!(emu.memory[emu.addr_reg], 2);
        assert_eq!(emu.memory[emu.addr_reg + 1], 3);
        assert_eq!(emu.memory[emu.addr_reg + 2], 7);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_f_55() {
        // reg_dump
        let mut emu = Chip8::init();

        // Init
        emu.addr_reg = 0x234;
        for i in 0..8 {
            emu.v[i] = (i + 0x57) as u8;
        }

        store_opcode(&mut emu, &[0xF755]);

        emu.emulate();
        for i in 0..8 {
            assert_eq!(emu.memory[emu.addr_reg + i], (i + 0x57) as u8);
        }
        assert_eq!(emu.addr_reg, 0x234);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    fn test_opcode_f_65() {
        // reg_load
        let mut emu = Chip8::init();

        // Init
        emu.addr_reg = 0x234;
        for i in 0..8 {
            emu.memory[emu.addr_reg + i] = (i + 0x57) as u8;
        }

        store_opcode(&mut emu, &[0xF765]);

        emu.emulate();
        for i in 0..8 {
            assert_eq!(emu.v[i], (i + 0x57) as u8);
        }
        assert_eq!(emu.addr_reg, 0x234);
        assert_eq!(emu.pc, PC_START + 2);
    }

    #[test]
    #[should_panic(expected = "Unknown opcode: 0xF777!")]
    fn test_opcode_f_unknown() {
        let mut emu = Chip8::init();

        store_opcode(&mut emu, &[0xF777]);

        emu.emulate();
    }
}
