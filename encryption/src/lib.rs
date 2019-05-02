extern crate libc;
#[macro_use]
extern crate logger;
extern crate memory_model;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use memory_model::{GuestAddress, GuestMemory, GuestMemoryError};
use serde::de::{Deserialize, Deserializer, Error};
use serde::ser::{Serialize, Serializer};
use std::io::{self, Read, Seek, Write};
use std::u8::MAX as U8_MAX;
use std::u8::MIN as U8_MIN;

//mod ffi;

#[derive(Debug)]
pub enum EncryptionError {
    /// Failure in accessing the block device
    IOError(io::Error),
    /// Failure in accessing the memory
    MemoryError(GuestMemoryError),
}

///The algorithm used for encryption
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum EncryptionAlgorithm {
    /// Advanced Encryption Standard with 256 bits key length, that uses Galois/Counter mode
    /// of operation
    AES256GCM,
}

/// Use this structure to set up the parameters used for encryption and decryption of data.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EncryptionDescription {
    /// Initialization vector. It is an arbitrary number that is used along with
    /// a secret key for data encryption.
    #[serde(deserialize_with = "deserialize")]
    #[serde(serialize_with = "serialize")]
    pub iv: Vec<u8>,
    /// The key used for data encryption
    #[serde(deserialize_with = "deserialize")]
    #[serde(serialize_with = "serialize")]
    pub key: Vec<u8>,
    /// Optional additional authenticated data.It is used as an integrity check.
    #[serde(deserialize_with = "deserialize")]
    #[serde(serialize_with = "serialize")]
    pub aad: Vec<u8>,
    ///The algorithm used for data encryption
    pub algorithm: EncryptionAlgorithm,
}

pub fn parse_str<S>(s: &S) -> Result<Vec<u8>, &str>
where
    S: AsRef<str> + ?Sized,
{
    if s.as_ref().len() % 2 == 1 {
        return Err(s.as_ref());
    }
    let mut bytes = Vec::with_capacity(s.as_ref().len() / 2);
    for i in (0..s.as_ref().len()).step_by(2) {
        bytes.push(u8::from_str_radix(&s.as_ref()[i..i + 2], 16).map_err(|_| s.as_ref())?);
    }
    Ok(bytes)
}

pub fn to_string(byte_array: &Vec<u8>) -> String {
    let mut hex_string = String::new();
    for i in 0..byte_array.len() {
        hex_string += &format!("{:02x}", byte_array[i]);
    }
    hex_string
}

fn serialize<S>(hex_array: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    to_string(hex_array).serialize(serializer)
}

fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    parse_str(&s).map_err(|_| D::Error::custom("The provided string is invalid."))
}

pub fn decrypt<T: Seek + Read + Write>(
    disk: &mut T,
    mem: &GuestMemory,
    data_addr: &mut GuestAddress,
    data_len: usize,
    iv: Vec<u8>,
    key: Vec<u8>,
) -> Result<usize, EncryptionError> {
    let mut initial_buffer = vec![0u8; data_len];
    let mut final_buffer = Vec::new();
    let mut bytes_count: usize = 0;

    while bytes_count < data_len {
        let _ = match disk.read(&mut initial_buffer) {
            Ok(len) => {
                for i in 0..len {
                    match initial_buffer[i] {
                        U8_MAX => final_buffer.push(U8_MIN),
                        _ => final_buffer.push(initial_buffer[i] + 1),
                    }
                }
                mem.write_slice_at_addr(&final_buffer, *data_addr)
                    .map_err(|e| EncryptionError::MemoryError(e))?;
                *data_addr = data_addr.checked_add(len as usize).unwrap();
                bytes_count += len as usize;
            }
            Err(e) => return Err(EncryptionError::IOError(e)),
        };
    }

    Ok(bytes_count)
}

pub fn encrypt<T: Seek + Read + Write>(
    disk: &mut T,
    mem: &GuestMemory,
    data_addr: &mut GuestAddress,
    data_len: usize,
    iv: Vec<u8>,
    key: Vec<u8>,
) -> Result<usize, EncryptionError> {
    let mut initial_buffer = vec![0u8; data_len];
    let mut final_buffer = Vec::new();
    let mut bytes_count: usize = 0;
    mem.read_slice_at_addr(&mut initial_buffer, *data_addr)
        .map_err(|e| EncryptionError::MemoryError(e))?;
    for i in 0..data_len {
        match initial_buffer[i] {
            U8_MIN => final_buffer.push(U8_MAX),
            _ => final_buffer.push(initial_buffer[i] - 1),
        }
    }
    while bytes_count < data_len {
        let _ = match disk.write(&mut final_buffer) {
            Ok(len) => {
                bytes_count += len as usize;
            }
            Err(e) => return Err(EncryptionError::IOError(e)),
        };
    }
    *data_addr = data_addr.checked_add(bytes_count).unwrap();
    Ok(bytes_count)
}

#[cfg(test)]
mod tests {

    extern crate serde_json;

    use super::*;

    #[test]
    fn test_encryption_parameter_str() {
        // odd length string
        assert!(parse_str("01234567890123456").is_err());

        // invalid hex
        assert!(parse_str("x123456789012345").is_err());

        let bytes = parse_str("0123456789012345").unwrap();
        assert_eq!(bytes, [0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45]);
    }

    #[test]
    fn test_encr_desc_serialization_and_deserialization() {
        let json = r#"{
            "iv":"0123456789012345",
            "key":"01234567890123456789012345678901",
            "aad":"012345",
            "algorithm":"AES256GCM"
        }"#;

        let encr_desc: EncryptionDescription =
            serde_json::from_str(json).expect("Deserialization failed");

        assert_eq!(
            encr_desc.iv,
            [0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45]
        );
        assert_eq!(
            encr_desc.key,
            [
                0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67,
                0x89, 0x01
            ]
        );
        assert_eq!(encr_desc.aad, [0x01, 0x23, 0x45]);

        let s = serde_json::to_string(&encr_desc).expect("Serialization failed.");

        let json = r#"{"iv":"0123456789012345","key":"01234567890123456789012345678901","aad":"012345","algorithm":"AES256GCM"}"#;
        assert_eq!(s, json);
    }
}
