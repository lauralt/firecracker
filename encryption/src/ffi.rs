extern crate libc;

use libc::{c_int, uint32_t, uint8_t, uint64_t};
use std::default::Default;
use std::ptr;

const EVP_MAX_IV_LENGTH : usize = 16;
const EVP_MAX_BLOCK_LENGTH : usize = 32;

#[repr(C)]
pub struct EvpCipherCtx {
    pub cipher: *const EvpCipher,
    //const EVP_CIPHER *cipher;
    pub engine: *mut Engine,
    //ENGINE *engine;             /* functional reference if 'cipher' is
    //   * ENGINE-provided */
    pub encrypt: c_int,
//int encrypt;                /* encrypt or decrypt */

    pub buf_len: c_int,
    //  int buf_len;                /* number we have left */
    pub oiv: [uint8_t; EVP_MAX_IV_LENGTH],

    pub iv: [uint8_t; EVP_MAX_IV_LENGTH],

    pub buf: [uint8_t; EVP_MAX_BLOCK_LENGTH],

    pub num: c_int,

    // pub basic_cpuid: [[uint32_t, ..4u], ..MAX_CPUID_LEVEL],
//unsigned char oiv[EVP_MAX_IV_LENGTH]; /* original iv */
//unsigned char iv[EVP_MAX_IV_LENGTH]; /* working iv */
//unsigned char buf[EVP_MAX_BLOCK_LENGTH]; /* saved partial block */
//int num;                    /* used by cfb/ofb/ctr mode */
    /* FIXME: Should this even exist? It appears unused */
    pub app_data: (),

    pub key_len: c_int,
    pub flags: uint64_t,
    pub cipher_data: (),
    pub final_used: c_int,
    pub block_mask: c_int,
    pub final_var: [uint8_t; EVP_MAX_BLOCK_LENGTH]
}
//void *app_data;             /* application stuff */
//int key_len;                /* May change for variable length cipher */
//unsigned long flags;        /* Various flags */
//void *cipher_data;          /* per EVP data */
//int final_used;
//int block_mask;
//unsigned char final[EVP_MAX_BLOCK_LENGTH]; /* possible final block */
//}

//pub struct EvpCipherCtx  {
//    pub data: *mut uint8_t,
//    pub size: uint32_t,
//    pub allocated: uint32_t,
//    pub mlocked: u32,
//}


#[repr(C)]
pub struct EvpCipher;

#[repr(C)]
pub struct Engine;

//pub struct evp_cipher_st {
//    pub nid: uint32_t,
//   // int nid;
//   pub blockSize: uint32_t,
//   // int block_size;
//    /* Default value for variable length ciphers */
//   pub keyLen: uint32_t,
//   // int key_len;
//   // int iv_len;
//   pub ivLen: uint32_t,
//    /* Various flags */
//    pub flags: uint64_t,
// //   unsigned long flags;
//    /* init key */
//    pub init: fn (ctx: *mut EvpCipherCtx, key: *const uint8_t, iv: *const uint8_t, enc: uint32_t) ->c_int,
//
//
//
//
//    int (*init) (EVP_CIPHER_CTX *ctx, const unsigned char *key,
//const unsigned char *iv, int enc);
///* encrypt/decrypt data */
//int (*do_cipher) (EVP_CIPHER_CTX *ctx, unsigned char *out,
//const unsigned char *in, size_t inl);
///* cleanup ctx */
//int (*cleanup) (EVP_CIPHER_CTX *);
///* how big ctx->cipher_data needs to be */
//int ctx_size;
///* Populate a ASN1_TYPE with parameters */
//int (*set_asn1_parameters) (EVP_CIPHER_CTX *, ASN1_TYPE *);
///* Get parameters from a ASN1_TYPE */
//int (*get_asn1_parameters) (EVP_CIPHER_CTX *, ASN1_TYPE *);
///* Miscellaneous operations */
//int (*ctrl) (EVP_CIPHER_CTX *, int type, int arg, void *ptr);
///* Application data */
//void *app_data;
//} /* EVP_CIPHER */ ;











//struct engine_st {
//const char *id;
//const char *name;
//const RSA_METHOD *rsa_meth;
//const DSA_METHOD *dsa_meth;
//const DH_METHOD *dh_meth;
//const EC_KEY_METHOD *ec_meth;
//const RAND_METHOD *rand_meth;
///* Cipher handling is via this callback */
//ENGINE_CIPHERS_PTR ciphers;
///* Digest handling is via this callback */
//ENGINE_DIGESTS_PTR digests;
///* Public key handling via this callback */
//ENGINE_PKEY_METHS_PTR pkey_meths;
///* ASN1 public key handling via this callback */
//ENGINE_PKEY_ASN1_METHS_PTR pkey_asn1_meths;
//ENGINE_GEN_INT_FUNC_PTR destroy;
//ENGINE_GEN_INT_FUNC_PTR init;
//ENGINE_GEN_INT_FUNC_PTR finish;
//ENGINE_CTRL_FUNC_PTR ctrl;
//ENGINE_LOAD_KEY_PTR load_privkey;
//ENGINE_LOAD_KEY_PTR load_pubkey;
//ENGINE_SSL_CLIENT_CERT_PTR load_ssl_client_cert;
//const ENGINE_CMD_DEFN *cmd_defns;
//int flags;
///* reference count on the structure itself */
//CRYPTO_REF_COUNT struct_ref;
///*
// * reference count on usability of the engine type. NB: This controls the
// * loading and initialisation of any functionality required by this
// * engine, whereas the previous count is simply to cope with
// * (de)allocation of this structure. Hence, running_ref <= struct_ref at
// * all times.
// */
//int funct_ref;
///* A place to store per-ENGINE data */
//CRYPTO_EX_DATA ex_data;
///* Used to maintain the linked-list of engines. */
//struct engine_st *prev;
//struct engine_st *next;
//};
//
//impl Default for EvpCipherCtx {
//    fn default() -> EvpCipherCtx {
//        EvpCipherCtx {
//            data: ptr::null_mut(),
//            size: 0,
//            allocated: 0,
//            mlocked: 0,
//        }
//    }
//}

//EVP_CIPHER_CTX *EVP_CIPHER_CTX_new(void);

//int EVP_EncryptInit_ex(EVP_CIPHER_CTX *ctx,
//const EVP_CIPHER *cipher, ENGINE *impl,
//const unsigned char *key,
//const unsigned char *iv);

#[link(name = "crypto")]
#[link(name = "ssl")]
extern "C" {

    pub fn EVP_CIPHER_CTX_new() -> *mut EvpCipherCtx;
    pub fn EVP_EncryptInit_ex(ctx: *mut EvpCipherCtx,
                               cipher: *const EvpCipher,
                               implem: *mut Engine,
                                key: *mut uint8_t,
                                iv: *mut uint8_t) -> c_int;
    pub fn EVP_aes_256_xts() -> *mut EvpCipher;

//const EVP_CIPHER *EVP_aes_256_xts(void);

//    pub fn s2n_blob_init(blob: *mut S2nBlob, data: *mut uint8_t, size: uint32_t) -> c_int;
//    pub fn s2n_blob_zero(blob: *mut S2nBlob) -> c_int;

}

#[cfg(test)]
mod tests {
    use super::*;

//    #[test]
//    fn test_s2n() {
//        let mut blob: S2nBlob = Default::default();
//        let r = unsafe { s2n_blob_zero(&mut blob) };
//        assert_eq!(r, 0);
//
//        let size: uint32_t = 8;
//        let mut data = vec![0x01u8, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45];
//        let raw_ptr = data.as_mut_ptr();
//
//        let d = unsafe { s2n_blob_init(&mut blob, raw_ptr, size) };
//        assert_eq!(d, 0);
//        assert_eq!(blob.size, size);
//        assert_eq!(blob.data, raw_ptr);
//        unsafe {
//            for i in 0..size as usize {
//                assert_eq!(*(blob.data.offset(i as isize)), data[i]);
//            }
//        }
//    }

        #[test]
    fn test_openssl() {
      //  let mut blob: S2nBlob = Default::default();
//        let r = unsafe { s2n_blob_zero(&mut blob) };
//        assert_eq!(r, 0);

        let data_len: uint32_t = 512;
        let mut key = vec![0x01u8, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89,
                           0x01u8, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89,
                           0x01u8, 0x23];
        let raw_key_ptr = key.as_mut_ptr();

        let mut iv = vec![0x01u8, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45];
        let raw_iv_ptr = iv.as_mut_ptr();

        assert_eq!(key.len(), 32);

        let mut bytes = vec![0u8; data_len as usize];

        let mut ciphertext : Vec<u8> = Vec::new();
        let mut decryptedtext : Vec<u8> = Vec::new();
        let raw_cipher_ptr = ciphertext.as_mut_ptr();
        let mut ctx = unsafe { EVP_CIPHER_CTX_new() };
        unsafe { EVP_EncryptInit_ex(ctx, EVP_aes_256_xts(), ptr::null_mut(), raw_key_ptr, raw_iv_ptr) };

   //     ciphertext_len = encrypt (plaintext, strlen ((char *)plaintext), key, iv, ciphertext);



//        let d = unsafe { s2n_blob_init(&mut blob, raw_ptr, size) };
//        assert_eq!(d, 0);
//        assert_eq!(blob.size, size);
//        assert_eq!(blob.data, raw_ptr);
//        unsafe {
//            for i in 0..size as usize {
//                assert_eq!(*(blob.data.offset(i as isize)), data[i]);
//            }

    }

}