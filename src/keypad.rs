use sdl2::keyboard::Keycode;

use crate::{FONT, KEYPAD_SIZE};




pub struct Keypad {
  keys: [bool; KEYPAD_SIZE]
}


impl Keypad {
  pub fn new() -> Self {
    Self {
      keys: [false; KEYPAD_SIZE]
    }
  }


  pub fn on_key_press(&mut self, i: usize, pressed: bool) {
    self.keys[i] = pressed;
  }

  pub fn get_keys(&self) -> &[bool] {
    &self.keys
  }

  pub fn get_font(&self) -> &[u8] {
    &FONT
  }

  pub fn key_to_btn(&self, key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 =>    Some(0x1),
        Keycode::Num2 =>    Some(0x2),
        Keycode::Num3 =>    Some(0x3),
        Keycode::Num4 =>    Some(0xC),
        Keycode::Q =>       Some(0x4),
        Keycode::W =>       Some(0x5),
        Keycode::E =>       Some(0x6),
        Keycode::R =>       Some(0xD),
        Keycode::A =>       Some(0x7),
        Keycode::S =>       Some(0x8),
        Keycode::D =>       Some(0x9),
        Keycode::F =>       Some(0xE),
        Keycode::Z =>       Some(0xA),
        Keycode::X =>       Some(0x0),
        Keycode::C =>       Some(0xB),
        Keycode::V =>       Some(0xF),
        _ =>                None,
    }
}
}