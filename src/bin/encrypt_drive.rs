use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::u8::MAX as U8_MAX;
use std::u8::MIN as U8_MIN;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut f = File::open(&args[1])?;
    let mut dst = File::create(&args[2])?;
    let mut buffer = Vec::new();
    let mut buffer_encr = Vec::new();

    f.read_to_end(&mut buffer)?;
    for i in 0..buffer.len() {
        match buffer[i] {
            U8_MIN => buffer_encr.push(U8_MAX),
            _ => buffer_encr.push(buffer[i] - 1),
        }
    }
    dst.write(&mut buffer_encr)?;
    Ok(())
}
