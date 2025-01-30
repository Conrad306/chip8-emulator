use rand::random;
use crate::{ display::Display, keypad::Keypad, FONT, FONT_SIZE, KEYPAD_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};

const STACK_SIZE: usize = 16;
const RAM_SIZE: usize = 4096;


const VARIABLE_REGISTER: usize = 16;

const START_ADDRESS: u16 = 0x200;

pub struct Emulator {
  counter: u16,
  ram: [u8; RAM_SIZE], 
  pub screen: Display,
  var_reg: [u8; VARIABLE_REGISTER],
  idx_reg: u16, 
  stack: [u16; STACK_SIZE],
  stack_ptr: u16,
  delay_timer: u8,
  sound_timer: u8, 
  pub keys: Keypad
}


impl Emulator {
  pub fn new() -> Self {
    let mut emulator =  Self {
      counter: START_ADDRESS, 
      ram: [0; RAM_SIZE], 
      screen: Display::new(), 
      var_reg: [0; VARIABLE_REGISTER], 
      idx_reg: 0, 
      stack: [0; STACK_SIZE], 
      stack_ptr: 0,
      delay_timer: 0, 
      sound_timer: 0,
      keys: Keypad::new()
    };

    emulator.ram[..FONT_SIZE].copy_from_slice(&FONT);

    emulator
  }


  pub fn push(&mut self, value: u16) {
    self.stack[self.stack_ptr as usize] = value;
    self.stack_ptr += 1;
  }
  pub fn pop(&mut self) -> u16 {
    self.stack_ptr -= 1; 
    self.stack[self.stack_ptr as usize]
  }

  pub fn cycle(&mut self) {
    let opcode = self.fetch();
    self.decode(opcode);
  }
  
  pub fn load(&mut self, data: &[u8]) {
    let start = START_ADDRESS as usize;
    let end = (START_ADDRESS as usize) + data.len();
    self.ram[start..end].copy_from_slice(data);
  }

  fn fetch(&mut self) -> u16 {
    let first_byte = self.ram[self.counter as usize] as u16;
    let second_byte = self.ram[(self.counter + 1) as usize] as u16; // "two successive bytes" 
    let opcode = (first_byte << 8) | second_byte;
    self.counter += 2;
    opcode
  }

  fn decode(&mut self, opcode: u16) {
    // split the addr by shifting each bit
    let first = (opcode & 0xF000) >> 12;
    let second = (opcode & 0x0F00) >> 8;
    let third = (opcode & 0x00F0) >> 4;
    let fourth = opcode & 0x000F;


    match (first, second, third, fourth) {
      (0, 0, 0, 0) => return, 
      (0, 0, 0xE, 0) => {
        self.screen.clear();
      },
      (0, 0, 0xE, 0xE) => {
        // return from subroutine ("Set PC to NNN")
        self.counter = self.pop();
      }, 
      (1, _, _, _) => {
        // Jump to given addr
        let nnn = opcode & 0x0FFF;
        self.counter = nnn;
      }, 
      (2, _, _, _) => {
        // enter subroutine
        let nnn = opcode & 0x0FFF;
        self.push(self.counter);
        self.counter = nnn;
      }, 
      (3, _, _, _) => {
        // Skip if VX == 0xNN
        let x = second as usize;
        let nn = (opcode & 0xFF) as u8;
        if self.var_reg[x] == nn {
          self.counter += 2;
        }

      }
      (4, _, _, _) => {
        // Skip if VX != 0xNN
        let x = second as usize;
        let nn = (opcode & 0xFF) as u8;
        if self.var_reg[x] != nn {
          self.counter += 2;
        }
      }, 
      (5, _, _, 0) => {
        // Skip if VX == VY
        let x = second as usize;
        let y = third as usize;
        if self.var_reg[x] == self.var_reg[y]  {
          self.counter += 2;
        }
      }, 
      (6, _, _, _) => {
        // vx = 0xnn
        let x = second as usize;
        let nn = (opcode & 0xFF) as u8;
        self.var_reg[x] = nn;
      }, 
      (7, _, _, _) => {
        // vx += 0xnn (no carry flag)
        let x: usize = second as usize;
        let nn = (opcode & 0xFF) as u8;
        self.var_reg[x] = self.var_reg[x].wrapping_add(nn); // Use wrapping add to prevent bit overflow (carry flag)
      },
      (8, _, _, 0) => {
        // vx = vy
        let x = second as usize;
        let y = third as usize;
        self.var_reg[x] = self.var_reg[y];
      },
      (8, _, _, 1) => {
        // vx |= vy
        let x = second as usize;
        let y = third as usize;
        self.var_reg[x] |= self.var_reg[y];
      },
      (8, _, _, 2) => {
        // vx &= vy
        let x = second as usize;
        let y = third as usize;
        self.var_reg[x] &= self.var_reg[y];
      },
      (8, _, _, 3) => {
        // vx ^= vy
        let x = second as usize;
        let y = third as usize;
        self.var_reg[x] ^= self.var_reg[y];
      },
      (8, _, _, 4) => {
        // vx += vy
        let x = second as usize;
        let y = third as usize;

        let (vx, carry) = self.var_reg[x].overflowing_add(self.var_reg[y]);

        // handle carry
        let vf = if carry { 1 } else { 0 };
        
        self.var_reg[0xF] = vf;
        self.var_reg[x] = vx; 
      },
      (8, _, _, 5) => {
        // vx -= vy
        let x = second as usize;
        let y = third as usize;

        let (vx, borrow) = self.var_reg[x].overflowing_sub(self.var_reg[y]);

        // handle carry
        let vf = if borrow { 0 } else { 1 };
        
        self.var_reg[0xF] = vf;
        self.var_reg[x] = vx; 
      },
      (8, _, _, 6) => {
        // vx >>= 1
        let x = second as usize;
        // handle dropped bit
        let dropped_bit = self.var_reg[x] & 1;
        self.var_reg[x] >>= 1;
        self.var_reg[0xF] = dropped_bit;
      },
      (8, _, _, 7) => {
        // vx -= vy
        let x = second as usize;
        let y = third as usize;

        let (vx, borrow) = self.var_reg[y].overflowing_sub(self.var_reg[x]);

        // handle carry
        let vf = if borrow { 0 } else { 1 };

        self.var_reg[x] = vx; 
        self.var_reg[0xF] = vf;
      },
      (8, _, _, 0xE) => {
        // vx <<= 1
        let x = second as usize;
        let missing_bit = (self.var_reg[x] >> 7) & 1;
        self.var_reg[x] <<= 1;
        self.var_reg[0xF] = missing_bit;
      },
      (9, _, _, 0) => {
        // skip if vx != vy
        let x = second as usize;
        let y = third as usize;
        if self.var_reg[x] != self.var_reg[y] {
          self.counter += 2;
        }
      },
      (0xA, _, _, _) => {
        // I = nnn
        let nnn = opcode & 0xFFF; 
        self.idx_reg = nnn;
      },
      (0xB, _, _, _) => {
        // Jump to V0 + 0xNNN
        let nnn: u16 = opcode & 0xFFF; 
        self.counter = (self.var_reg[0] as u16) + nnn;
      },
      (0xC, _, _, _) => {
        // VX = rand() & 0xnn
        let x = second as usize;
        let nn = (opcode & 0xFF) as u8;
        let rand: u8 = random::<u8>(); 
        self.var_reg[x] =  rand & nn;
      },
      (0xD, _, _, _) => {
        // Draw sprite 
        // "the horizontal x coordinate in vx... the Y coordinate in VY"
        let coord_x = self.var_reg[second as usize] as u16;
        let coord_y = self.var_reg[third as usize] as u16;
        
        let rows = fourth;

        let mut flipped = false;

        for y_line in 0..rows {
          let addr = self.idx_reg + y_line as u16;
          let pixels = self.ram[addr as usize];

          for x_line in 0..8 {
            const MASK_BIT: u8 = 0b10000000;
            // check if the pixel is active
            if(pixels & (MASK_BIT >> x_line)) != 0 {
              // "the coordinates are modulo the size of the display (when counting from 0)."
              let x = (coord_x + x_line) as usize % SCREEN_WIDTH;
              let y = (coord_y + y_line) as usize % SCREEN_HEIGHT;

              let i = x + SCREEN_WIDTH * y;
              // Flip the bit on if it isn't.
              flipped |= self.screen.get_screen()[i];
              self.screen.set_screen(i, self.screen.get_screen()[i] ^ true);
            }
          }
        }

        if flipped {
          self.var_reg[0xF] = 1;
        } else {
          self.var_reg[0xF] = 0;
        }
      },
      (0xE, _, 9, 0xE) => {
        // Skip if key pressed
        let x = second as usize;
        let vx = self.var_reg[x]; 
        if self.keys.get_keys()[vx as usize] {
          self.counter += 2;
        }
      },
      (0xE, _, 0xA, 1) => {
        // Skip if key not pressed
        let x = second as usize;
        let vx = self.var_reg[x];
        if !self.keys.get_keys()[vx as usize] {
          self.counter += 2;
        }
      },
      (0xF, _, 0, 7) => {
        // VX = delay timer
        let x = second as usize;
        self.var_reg[x] = self.delay_timer;
      },
      (0xF, _, 0, 0xA) => {
        // Wait for key press (blocking)
        let x = second as usize;
        let mut pressed = false; 
        // iterate over all keys
        for i in 0..self.keys.get_keys().len() {
          // if the key exists, they've pressed something.
          if self.keys.get_keys()[i] {
            self.var_reg[x] = i as u8; 
            pressed = true; 
            break;
          }
        }
        if !pressed {
          // Retry code
          self.counter -= 2; 
        }
      }, 
      (0xF, _, 1, 5) => {
        // Delay timer = vx
        let x = second as usize;
        self.delay_timer = self.var_reg[x];
      },
      (0xF, _, 1, 8) => {
        // Sound timer = vx
        let x = second as usize;
        self.sound_timer = self.var_reg[x];
      }, 
      (0xF, _, 1, 0xE) => { 
        // I += vx
        let x = second as usize;
        self.idx_reg = self.idx_reg.wrapping_add(self.var_reg[x] as u16);
      },
      (0xF, _, 2, 9) => {
        // Set I to addr in font character of vx;
        let x = second as usize;
        // mult by 5, since font addr points are v_reg[x] * 5
        let addr = self.var_reg[x] as u16;
        self.idx_reg = addr * 5;
      },
      (0xF, _, 3, 3) => {
        // Store binary-coded-decimal (BCD) of VX into I
        let x = second as usize; 
        let vx = self.var_reg[x] as f32;

        // get each decimal digit
        let hund = (vx / 100.0).floor() as u8;
        let tens = ((vx / 10.0) % 10.0).floor() as u8; 
        let ones = (vx % 10.0) as u8;
        
        self.ram[self.idx_reg as usize] = hund;
        self.ram[(self.idx_reg + 1) as usize] = tens;
        self.ram[(self.idx_reg + 2) as usize] = ones;
      },
      (0xF, _, 5, 5) => {
        // Put v0-vx into I
        let x = second as usize;
        let idx = self.idx_reg as usize;
        for i in 0..=x {
            self.ram[idx + i] = self.var_reg[i];
        }
      },
      (0xF, _, 6, 5) => {
        // the opposite: Load I into v0-vx
        let x = second as usize;
        let idx = self.idx_reg as usize;
        for i in 0..=x {
            self.var_reg[i] = self.ram[idx + i];
        }
      }
      // Handle default
      (_, _, _, _) => unimplemented!("Opcode unimplemented: {:#04x}", opcode),
    }
  }



  /** Decrement every frame */
  pub fn timer(&mut self) {
    if self.delay_timer > 0 {
      self.delay_timer -= 1
    }


    /** Beep when the sound timer hits 1 */
    if self.sound_timer > 0 {
      if self.sound_timer == 1 {
        // beep here
      }
      self.sound_timer -= 1;
    }
  }


}