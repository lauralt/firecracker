extern crate libc;
#[macro_use]
extern crate logger;
extern crate memory_model;
extern crate openssl;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use memory_model::{GuestAddress, GuestMemory, GuestMemoryError};
use openssl::error::ErrorStack;
use openssl::symm::decrypt as openssl_decrypt;
use openssl::symm::encrypt as openssl_encrypt;
use openssl::symm::Cipher;
use serde::de::{Deserialize, Deserializer, Error};
use serde::ser::{Serialize, Serializer};
use std::io::{self, Read, Seek, Write};

#[derive(Debug)]
pub enum EncryptionError {
    /// Failure in accessing the block device
    IOError(io::Error),
    /// Failure in accessing the memory
    MemoryError(GuestMemoryError),
    /// Failure in encrypting/decrypting with the cipher
    OpensslError(ErrorStack),
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
    encryption_description: &EncryptionDescription,
) -> Result<usize, EncryptionError> {
    let mut initial_buffer = vec![0u8; data_len];
    let mut final_buffer;
    let mut bytes_count: usize = 0;
    let cipher = Cipher::aes_256_xts();
    while bytes_count < data_len {
        let _ = match disk.read(&mut initial_buffer) {
            Ok(len) => {
                final_buffer = openssl_decrypt(
                    cipher,
                    &encryption_description.key,
                    Some(&encryption_description.iv),
                    &initial_buffer,
                )
                .map_err(|e| EncryptionError::OpensslError(e))?;
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
    encryption_description: &EncryptionDescription,
) -> Result<usize, EncryptionError> {
    let mut initial_buffer = vec![0u8; data_len];
    let mut final_buffer;
    let mut bytes_count: usize = 0;
    let cipher = Cipher::aes_256_xts();
    mem.read_slice_at_addr(&mut initial_buffer, *data_addr)
        .map_err(|e| EncryptionError::MemoryError(e))?;
    final_buffer = openssl_encrypt(
        cipher,
        &encryption_description.key,
        Some(&encryption_description.iv),
        &initial_buffer,
    )
    .map_err(|e| EncryptionError::OpensslError(e))?;
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
    use openssl::aes::{aes_ige, AesKey};
    use openssl::symm::Mode;

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
            "key":"0123456789012345678901234567890A",
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
                0x89, 0x0A
            ]
        );
        assert_eq!(encr_desc.aad, [0x01, 0x23, 0x45]);

        let s = serde_json::to_string(&encr_desc).expect("Serialization failed.");

        let json = r#"{"iv":"0123456789012345","key":"0123456789012345678901234567890a","aad":"012345","algorithm":"AES256GCM"}"#;
        assert_eq!(s, json);
    }

    //    #[test]
    //    fn test_openssl_aes_ige() {
    //        let key = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F";
    //        let plaintext = b"\x12\x34\x56\x78\x90\x12\x34\x56\x12\x34\x56\x78\x90\x12\x34\x56";
    //        let mut iv = vec![
    //            0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
    //            0x0E, 0x0F, 0x10u8, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
    //            0x1C, 0x1D, 0x1E, 0x1F,
    //        ];
    //
    //        let mut iv_2 = vec![
    //            0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
    //            0x0E, 0x0F, 0x10u8, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B,
    //            0x1C, 0x1D, 0x1E, 0x1F,
    //        ];
    //        let key_1 = AesKey::new_encrypt(key).unwrap();
    //
    //        let key_2 = AesKey::new_decrypt(key).unwrap();
    //        let mut output = [0u8; 16];
    //        aes_ige(plaintext, &mut output, &key_1, &mut iv, Mode::Encrypt);
    //        assert_eq!(
    //            output,
    //            *b"\xa6\xad\x97\x4d\x5c\xea\x1d\x36\xd2\xf3\x67\x98\x09\x07\xed\x32"
    //        );
    //        let mut output_2 = [0u8; 16];
    //        aes_ige(&output, &mut output_2, &key_2, &mut iv_2, Mode::Decrypt);
    //        assert_eq!(*plaintext, output_2);
    //    }
    //
    //    #[test]
    //    fn test_openssl_aes_xts() {
    //        use openssl::symm::Cipher;
    //
    //        let data_len: usize = 512;
    //        let cipher = Cipher::aes_256_xts();
    //        let key = vec![
    //            0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
    //            0x0E, 0x0F, 0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
    //            0x0C, 0x0D, 0x0E, 0x0F, 0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
    //            0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    //            0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    //        ];
    //        let iv = vec![
    //            0x00u8, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
    //            0x0E, 0x0F,
    //        ];
    //
    //        let mut initial_buffer = vec![0u8; data_len];
    //        for i in 0..data_len {
    //            initial_buffer[i] = 0;
    //        }
    //
    //        let encrypted_buffer =
    //            openssl::symm::encrypt(cipher, &key, Some(&iv), &initial_buffer).unwrap();
    //        let decrypted_buffer =
    //            openssl::symm::decrypt(cipher, &key, Some(&iv), &encrypted_buffer).unwrap();;
    //
    //        if decrypted_buffer != initial_buffer {
    //            println!("Computed: {:?}", initial_buffer);
    //            println!("Expected: {:?}", decrypted_buffer);
    //            if initial_buffer.len() != decrypted_buffer.len() {
    //                println!(
    //                    "Lengths differ: {} in computed vs {} expected",
    //                    initial_buffer.len(),
    //                    decrypted_buffer.len()
    //                );
    //            }
    //            panic!("test failure");
    //        }
    //        assert_eq!(initial_buffer, decrypted_buffer);
    //    }
}
