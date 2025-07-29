mod cpu;
mod video;
mod input;
mod consts;

use std::fs;
use std::env;
use std::io;

use cpu::Cpu;
use video::Video;
use input::Input;

use sdl2;

fn main() {
    let cfg = parse_args(env::args());

    let read_mem: Vec<u8> = match read_chip8_programm(&cfg.chip8_filepath) {
        Ok(t) => t,
        Err(e) => panic!("{}", e),
    };

    let sdl_context = sdl2::init().unwrap();

    let mut video = Video::new(&sdl_context, 680,320,"chip-8 emulator",
        (0, 0, 0),
        (255, 255, 255));

    let mut input = Input::new(&sdl_context);
    let mut cpu = Cpu::new(read_mem);

    loop {
        //trace_prompt(&cpu);
        cpu.tick(input.event_poll());
        
        if cpu.vmem_changed {
            match video.render_sprite_new(&cpu.vmem, 1, 1, 10) {
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            };
        }
        //clock_prev = std::time::Instant::now();

        //std::thread::sleep(std::time::Duration::new(0, 1_450_500 as u32));
        std::thread::sleep(std::time::Duration::new(0, 1000 as u32));
    }
}

fn parse_args(mut args: env::Args) -> Config {
    let prog_name = match args.next() {
        Some(arg) => arg,
        None => panic!("Unreachable"),
    };
    
    let chip8_file = match args.next() {
        Some(arg) => arg,
        None => panic!("usage: {} chip-8-filename.ch8", prog_name),
    };
    Config::new(chip8_file)
}

#[derive(Debug)]
struct Config {
    chip8_filepath: String,
}

impl Config {
    fn new(filename: String) -> Self {
        Self {
            chip8_filepath: filename,
        }
    }
}

fn read_chip8_programm(filepath: &str) -> Result<Vec<u8>, std::io::Error>{
    Ok(fs::read(filepath)?)
}

#[allow(dead_code)]
fn trace_prompt(cpu: &Cpu) {
    println!("{}", cpu);
    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer).unwrap();

    match buffer.as_str().trim() {
        "d" => println!("{}", cpu),
        "help" => println!("Display CPU: 'd', otherwise press Enter"), 
        _ => (),
    };
}
