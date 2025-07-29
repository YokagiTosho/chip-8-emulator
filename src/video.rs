use sdl2;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::Rect;

use crate::consts::*;

pub struct Video {
    bg_color: Color,
    fg_color: Color, 
    canvas: Canvas<Window>,
}

impl Video {
    pub fn new(ctx: &sdl2::Sdl,
               width: u32,
               height: u32,
               title: &str,
               bg: (u8, u8, u8),
               fg: (u8, u8, u8)) -> Self 
    {
        let window = ctx
            .video()
            .unwrap()
            .window(&title, width, height)
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(bg.0, bg.1, bg.2));
        canvas.clear();
        canvas.present();
        Self { 
            canvas,
            bg_color: Color::RGB(bg.0, bg.1, bg.2),
            fg_color: Color::RGB(fg.0, fg.1, fg.2),
        }
    }
    
    pub fn render_sprite_new(&mut self,
        sprite: &[[u8; SCR_HEIGHT]; SCR_WIDTH],
        width: u32,
        height: u32,
        scale: u32) -> Result<(), String>
    {
        for i in 0..SCR_HEIGHT {
            for j in 0..SCR_WIDTH {
                if sprite[j][i] == 1 {
                    self.canvas.set_draw_color(self.fg_color);
                    self.canvas.fill_rect(Rect::new(
                            j as i32*scale as i32,
                            i as i32*scale as i32,
                            width*scale,
                            height*scale))?;
                } else {
                    self.canvas.set_draw_color(self.bg_color);
                    self.canvas.fill_rect(Rect::new(
                            j as i32*scale as i32,
                            i as i32*scale as i32,
                            width*scale,
                            height*scale))?;
                }
            }
        }
        self.canvas.present();
        Ok(())
    }
}
