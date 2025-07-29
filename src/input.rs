use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::process;

/*
 * 1 2 3 4 
 * q w e r
 * a s d f
 * z x c v
 * 1 => 0x1,
 * 2 => 0x2,
 * 3 => 0x3,
 * 4 => 0x4,
 * q => 0x5,
 * w => 0x6,
 * e => 0x7,
 * r => 0x8,
 * a => 0x9,
 * s => 0x0,
 * d => 0xA,
 * f => 0xB,
 * z => 0xC,
 * x => 0xD,
 * c => 0xE,
 * v => 0xF,
*/

pub struct Input {
    events: sdl2::EventPump,
}

impl Input {
    pub fn new(ctx: &sdl2::Sdl) -> Self {
        Self {
            events: ctx.event_pump().unwrap(),
        }
    }
    
    pub fn event_poll(&mut self) -> [bool; 16] {
        let mut keyboard_arr: [bool; 16] = [false; 16];

        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { 
                    keycode: Some(sdl2::keyboard::Keycode::Escape), ..
                }
                => {
                    println!("Exit(ESC) pressed");
                    process::exit(0);    
                },
                Event::KeyDown { 
                    keycode: Some(t), ..
                } => match t {
                    Keycode::Num1 => keyboard_arr[0x1] = true,
                    Keycode::Num2 => keyboard_arr[0x2] = true,
                    Keycode::Num3 => keyboard_arr[0x3] = true,
                    Keycode::Num4 => keyboard_arr[0x4] = true,
                    Keycode::Q    => keyboard_arr[0x5] = true,
                    Keycode::W    => keyboard_arr[0x6] = true,
                    Keycode::E    => keyboard_arr[0x7] = true,
                    Keycode::R    => keyboard_arr[0x8] = true,
                    Keycode::A    => keyboard_arr[0x9] = true,
                    Keycode::S    => keyboard_arr[0x0] = true,
                    Keycode::D    => keyboard_arr[0xA] = true,
                    Keycode::F    => keyboard_arr[0xB] = true,
                    Keycode::Z    => keyboard_arr[0xC] = true,
                    Keycode::X    => keyboard_arr[0xD] = true,
                    Keycode::C    => keyboard_arr[0xE] = true,
                    Keycode::V    => keyboard_arr[0xF] = true,
                    _ => (),
                },
                _ => (),
            };
        }
        keyboard_arr
    }
}
