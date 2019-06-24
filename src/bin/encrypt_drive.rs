/// For encrypting a drive before attaching it to a microvm, we should run
/// 'cargo run --bin encrypt_drive path_to_unencrypted_drive path_for_the_encrypted_drive'
extern crate encryption;
extern crate openssl;

use openssl::symm::{encrypt, Cipher};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, Result};
use std::mem::transmute;

use encryption::transform_u128_to_array_of_u8;

pub const SECTOR_SIZE: usize = 512;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut f = File::open(&args[1])?;
    let mut dst = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&args[2])?;
    let mut buffer = Vec::new();
    let mut buffer_encr;
    let cipher = Cipher::aes_256_xts();
    let key = vec![
        0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F, 0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
        0x0E, 0x0F, 0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
        0x0D, 0x0E, 0x0F, 0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
        0x0C, 0x0D, 0x0E, 0x0F,
    ];

    f.read_to_end(&mut buffer)?;
    let no_sectors = buffer.len() / SECTOR_SIZE;

    for no_sector in 0..no_sectors {
        let iv: [u8; 16] = transform_u128_to_array_of_u8(no_sector as u128);
        buffer_encr = encrypt(
            cipher,
            &key,
            Some(&iv),
            &buffer[no_sector * SECTOR_SIZE..(no_sector + 1) * SECTOR_SIZE],
        )
        .unwrap();
        dst.write(&mut buffer_encr)?;
        buffer_encr.clear();
    }
    Ok(())
}
