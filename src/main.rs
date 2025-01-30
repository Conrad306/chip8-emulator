use std::io::Read;
use std::{env, fs::File};

mod emulator;
mod display;
mod keypad;
use emulator::Emulator;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub const FONT_SIZE: usize = 80;
pub const KEYPAD_SIZE: usize = 16;
pub const FONT: [u8; FONT_SIZE] = [
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

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const SCALE: u32 = 15;
pub const TICKS_PER_FRAME: i8 = 10;

const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE; 
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE; 

fn main() {
  let args: Vec<_> = env::args().collect();
  if args.len() != 2 {
      println!("Usage: cargo run roms/game");
      return;
  }

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem
      .window("Chip8", WINDOW_WIDTH, WINDOW_HEIGHT)
      .position_centered()
      .opengl()
      .build()
      .unwrap();

  let mut canvas = window.into_canvas().present_vsync().build().unwrap();
  canvas.clear();
  canvas.present();

  let mut event_pump = sdl_context.event_pump().unwrap();

  let mut emulator = Emulator::new();

  let mut rom = File::open(&args[1]).expect("Unable to open file");
  let mut buffer = Vec::new();

  rom.read_to_end(&mut buffer).unwrap();
  emulator.load(&buffer);


  'init: loop {
      for evt in event_pump.poll_iter() {
          match evt {
              Event::Quit{..} | Event::KeyDown{keycode: Some(Keycode::Escape), ..}=> {
                  break 'init;
              },

              Event::KeyDown{keycode: Some(key), ..} => {
                  if let Some(k) =  emulator.keys.key_to_btn(key) {
                    emulator.keys.on_key_press(k, true);
                  }
              },
              Event::KeyUp{keycode: Some(key), ..} => {
                  if let Some(k) = emulator.keys.key_to_btn(key) {
                    emulator.keys.on_key_press(k, false);
                  }
              },
              _ => ()
          }
      }

      for _ in 0..TICKS_PER_FRAME {
        emulator.cycle();
      }
      
      emulator.timer();
      emulator.screen.draw(&mut canvas);
  }
}