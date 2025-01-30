use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

use crate::{ SCREEN_WIDTH, SCREEN_HEIGHT, SCALE };



pub struct Display  {
  screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT]
}

impl Display {
  pub fn new() -> Self {
    Self {
        screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT]
    }
  }

  pub fn clear(&mut self) {
    self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
  }

  pub fn set_screen(&mut self, index: usize, val: bool) {
      self.screen[index] = val;
  }


  pub fn get_screen(&self) -> &[bool] {
    &self.screen
  }

  pub fn draw(&mut self, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0,0));
    canvas.clear();


    canvas.set_draw_color(Color::RGB(0, 255, 0));
    // draw pxs
    for (i, pixel) in self.screen.iter().enumerate() {
      if *pixel {
        let x = (i % SCREEN_WIDTH) as u32;
        let y = (i / SCREEN_WIDTH) as u32;

        let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, 15, 15);
        canvas.fill_rect(rect).unwrap();
      }
    }
    canvas.present();
  }
}