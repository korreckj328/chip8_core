// chip 8 emulation library
// Jeremiah Korreck

use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const NUM_REGS: usize = 16;
const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80 //F
];

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
    pub beeping: bool,
}

impl Emu {
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn get_sound_state(&self) -> bool {
        self.beeping
    }

    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
            beeping: false,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        // fetch
        let op = self.fetch();

        //decode and execute

        self.execute(op);

    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            //nop
            (0,0,0,0) => return,
            //cls
            (0,0,0xE,0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            },
            //ret
            (0,0,0xE,0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            },
            // jmp nnn
            (1,_,_,_) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            },
            //call nnn
            (2,_,_,_) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            },
            //skip vx == nn
            (3,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            },
            // skip vx != nn
            (4,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            // skip vx = vy
            (5,_,_,0) => {
                let x =digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            },
            //vx = nn
            (6,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            },
            //vx += nn
            (7,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            },
            //vx = vy
            (8,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            },
            // vx |= vy
            (8,_,_,1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            },
            // vx &= vy
            (8,_,_,2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            },
            // vx ^= vy
            (8,_,_,3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            },
            // vx += vy
            (8,_,_,4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry {1} else {0};
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // vx -= vy
            (8,_,_,5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow {0} else {1};
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            //vx >>= 1
            (8,_,_,6) => {
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            },
            //vy - vx
            (8,_,_,7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow {0} else {1};
                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            },
            // vx <<= 1
            (8,_,_,0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            },
            //skip if vx != vy
            (9,_,_,0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            },
            // i = nnn
            (0xA,_,_,_) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            },
            // jump v0 + nnn
            (0xB,_,_,_) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            },
            //rand() & nn
            (0xC,_,_,_) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            },
            //draw sprite
            (0xD,_,_,_) => {
                // get (x,y) for the sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;

                let sprite_height = digit4;

                // keep track of if pixels are flipped
                let mut pixel_flipped = false;

                for y_line in 0..sprite_height {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // iterate over each column
                    for x_line in 0..8 {
                        // use a mask to fetch current pixels bit flip if 1
                        if (pixels & (0b10000000 >> x_line)) != 0 {
                            // sprites should wrap screen, apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            // get pixel's index
                            let idx = x + SCREEN_WIDTH * y;
                            // check if pixel is to be flipped
                            pixel_flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                // populate vf register
                if pixel_flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            // skip key press
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            // skip if key not pressed
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            //vx = dt
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            //wait until key
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    // redo op code so that it waits in a loop
                    self.pc -= 2;
                }
            },
            // dt = vx
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            // st = vx
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            // i += vx
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            },
            // i = font
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            // bcd
            // bcd converst from hex to dec
            // needs optimization
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;
                let hundreds = (vx/100.0).floor() as u8;
                let tens = ((vx/10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;
                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            },
            // store v0 - vx
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            },
            // load v0 - vx
            (0xF, _,6,5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx];
                }
            },
            (_,_,_,_) => unimplemented!("Unimplemented opcode: {}", op),

        }
    }

    pub fn get_display(& self) -> &[bool] {
        & self.screen
    }

    pub fn keypress(& mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(& mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                self.beeping = true;
            } else {
                self.beeping = false;
            }
            self.st -= 1;
        }
    }
}
