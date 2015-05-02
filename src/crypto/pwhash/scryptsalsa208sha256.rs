//! `crypto_pwhash_scryptsalsa208sha256`, a particular combination of Scrypt, Salsa20/8
//! and SHA-256
use ffi;
use std::ops::{Index, Range, RangeFrom, RangeFull, RangeTo};
use randombytes::randombytes_into;
use libc::{c_ulonglong, size_t};

pub const SALTBYTES: usize = ffi::crypto_pwhash_scryptsalsa208sha256_SALTBYTES;
pub const STRBYTES: usize = ffi::crypto_pwhash_scryptsalsa208sha256_STRBYTES;
pub const STRPREFIX: &'static str = ffi::crypto_pwhash_scryptsalsa208sha256_STRPREFIX;
pub const OPSLIMIT_INTERACTIVE: OpsLimit =
    OpsLimit(ffi::crypto_pwhash_scryptsalsa208sha256_OPSLIMIT_INTERACTIVE);
pub const MEMLIMIT_INTERACTIVE: MemLimit =
    MemLimit(ffi::crypto_pwhash_scryptsalsa208sha256_MEMLIMIT_INTERACTIVE);
pub const OPSLIMIT_SENSITIVE: OpsLimit =
    OpsLimit(ffi::crypto_pwhash_scryptsalsa208sha256_OPSLIMIT_SENSITIVE);
pub const MEMLIMIT_SENSITIVE: MemLimit =
    MemLimit(ffi::crypto_pwhash_scryptsalsa208sha256_MEMLIMIT_SENSITIVE);

/// `OpsLimit` represents the maximum number of computations to perform when
/// using the functions in this module.
///
/// A high `OpsLimit` will make the functions
/// require more CPU cycles
#[derive(Copy, Clone)]
pub struct OpsLimit(pub usize);

/// `MemLimit` represents the maximum amount of RAM that the functions in this
/// module will use, in bytes.
///
/// It is highly recommended to allow the functions to use
/// at least 16 megabytes.
#[derive(Copy, Clone)]
pub struct MemLimit(pub usize);

/// `Salt` used for password hashing
#[derive(Copy)]
pub struct Salt(pub [u8; SALTBYTES]);
newtype_clone!(Salt);
newtype_impl!(Salt, SALTBYTES);

/// `HashedPassword`is a password verifier generated from a password
///
/// A `HashedPassword` is zero-terminated, includes only ASCII characters and can
/// be conveniently stored into SQL databases and other data stores. No
/// additional information has to be stored in order to verify the password.
pub struct HashedPassword(pub [u8; STRBYTES]);
newtype_clone!(HashedPassword);
newtype_impl!(HashedPassword, STRBYTES);

/// `gen_salt()` randombly generates a new `Salt` for key derivation
///
/// THREAD SAFETY: `gen_salt()` is thread-safe provided that you have called
/// `sodiumoxide::init()` once before using any other function from sodiumoxide.
pub fn gen_salt() -> Salt {
    let mut salt = Salt([0; SALTBYTES]);
    {
        let Salt(ref mut sb) = salt;
        randombytes_into(sb);
    }
    salt
}

/// The `derive_key()` function derives a key from a password and a `Salt`
///
/// The computed key is stored into out.
///
/// `opslimit` represents a maximum amount of computations to perform. Raising
/// this number will make the function require more CPU cycles to compute a key.
///
/// `memlimit` is the maximum amount of RAM that the function will use, in
/// bytes. It is highly recommended to allow the function to use at least 16
/// megabytes.
///
/// For interactive, online operations, `OPSLIMIT_INTERACTIVE` and
/// `MEMLIMIT_INTERACTIVE` provide a safe base line for these two
/// parameters. However, using higher values may improve security.
///
/// For highly sensitive data, `OPSLIMIT_SENSITIVE` and `MEMLIMIT_SENSITIVE` can
/// be used as an alternative. But with these parameters, deriving a key takes
/// more than 10 seconds on a 2.8 Ghz Core i7 CPU and requires up to 1 gigabyte
/// of dedicated RAM.
///
/// The salt should be unpredictable. `gen_salt()` is the easiest way to create a `Salt`.
///
/// Keep in mind that in order to produce the same key from the same password,
/// the same salt, and the same values for opslimit and memlimit have to be
/// used.
///
/// The function returns `Some(key)` on success and `None` if the computation didn't
/// complete, usually because the operating system refused to allocate the
/// amount of requested memory.
/// #Example
/// ```
/// use sodiumoxide::crypto::secretbox::{Key, KEYBYTES};
/// use sodiumoxide::crypto::pwhash::{gen_salt, derive_key,
///                                   OPSLIMIT_INTERACTIVE,
///                                   MEMLIMIT_INTERACTIVE};
///
/// let passwd = "Correct Horse Battery Staple".as_bytes();
/// let salt = gen_salt();
/// let mut k = Key([0; KEYBYTES]);
/// {
///     let Key(ref mut kb) = k;
///     derive_key(kb, passwd, &salt,
///                OPSLIMIT_INTERACTIVE,
///                MEMLIMIT_INTERACTIVE).unwrap();
/// }
/// ```
pub fn derive_key(key: &mut [u8], passwd: &[u8], &Salt(ref sb): &Salt,
                  OpsLimit(opslimit): OpsLimit,
                  MemLimit(memlimit): MemLimit) -> Option<()> {
    if unsafe {
        ffi::crypto_pwhash_scryptsalsa208sha256(key.as_mut_ptr(),
                                                key.len() as c_ulonglong,
                                                passwd.as_ptr(),
                                                passwd.len() as c_ulonglong,
                                                sb,
                                                opslimit as c_ulonglong,
                                                memlimit as size_t)
    } == 0 {
        Some(())
    } else {
        None
    }
}

/// The `pwhash()` returns a `HashedPassword` which
/// includes:
///
/// - the result of a memory-hard, CPU-intensive hash function applied to the password
///   `passwd`
/// - the automatically generated salt used for the
///   previous computation
/// - the other parameters required to verify the password: opslimit and memlimit
///
/// `OPSLIMIT_INTERACTIVE` and `MEMLIMIT_INTERACTIVE` are safe baseline
/// values to use for `opslimit` and `memlimit`.
///
/// The function returns `Some(hashed_password)` on success and `None` if it didn't complete
/// successfully
/// #Example
/// ```
/// use sodiumoxide::crypto::pwhash::{pwhash, HashedPassword,
///                                   OPSLIMIT_INTERACTIVE, MEMLIMIT_INTERACTIVE};
/// let passwd = "Correct Horse Battery Staple".as_bytes();
/// let pwh = pwhash(passwd, OPSLIMIT_INTERACTIVE, MEMLIMIT_INTERACTIVE).unwrap();
/// let pwh_bytes = &pwh[..];
/// //store pwh_bytes somewhere
/// ```
pub fn pwhash(passwd: &[u8], OpsLimit(opslimit): OpsLimit,
              MemLimit(memlimit): MemLimit) -> Option<HashedPassword> {
    let mut out = HashedPassword([0; STRBYTES]);
    if unsafe {
        let HashedPassword(ref mut str_) = out;
        ffi::crypto_pwhash_scryptsalsa208sha256_str(str_,
                                                    passwd.as_ptr(),
                                                    passwd.len() as c_ulonglong,
                                                    opslimit as c_ulonglong,
                                                    memlimit as size_t)
    } == 0 {
        Some(out)
    } else {
        None
    }
}

/// `pwhash_verify()` verifies that the password `str_` is a valid password
/// verification string (as generated by `pwhash()`) for `passwd`
///
/// It returns `true` if the verification succeeds, and `false` on error.
/// #Example
/// ```
/// use sodiumoxide::crypto::pwhash::{pwhash, pwhash_verify, HashedPassword,
///                                   OPSLIMIT_INTERACTIVE, MEMLIMIT_INTERACTIVE};
/// let passwd = "Correct Horse Battery Staple".as_bytes();
/// // in reality we want to load the password hash from somewhere
/// // and we might want to create a `HashedPassword` from it using
/// // `HashedPassword::from_slice(pwhash_bytes).unwrap()`
/// let pwh = pwhash(passwd, OPSLIMIT_INTERACTIVE, MEMLIMIT_INTERACTIVE).unwrap();
/// assert!(pwhash_verify(&pwh, passwd));
/// ```
pub fn pwhash_verify(&HashedPassword(ref str_): &HashedPassword,
                     passwd: &[u8]) -> bool {
    unsafe {
        ffi::crypto_pwhash_scryptsalsa208sha256_str_verify(str_,
                                                           passwd.as_ptr(),
                                                           passwd.len() as c_ulonglong)
            == 0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_derive_key() {
        let mut kb = [0u8; 32];
        let salt = Salt([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
                         16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
        let pw = "Correct Horse Battery Staple".as_bytes();
        // test vector generated by using libsodium
        let kb_expected = [0xf1, 0xbb, 0xb8, 0x7c, 0x43, 0x36, 0x5b, 0x03,
                           0x3b, 0x9a, 0xe8, 0x3e, 0x05, 0xef, 0xad, 0x25,
                           0xdb, 0x8d, 0x83, 0xb8, 0x3d, 0xb1, 0xde, 0xe3,
                           0x6b, 0xdb, 0xf5, 0x4d, 0xcd, 0x3a, 0x1a, 0x11];
        derive_key(&mut kb, pw, &salt, OPSLIMIT_INTERACTIVE, MEMLIMIT_INTERACTIVE);
        assert_eq!(kb, kb_expected);
    }

    #[test]
    fn test_pwhash_verify() {
        use randombytes::randombytes;
        for i in (0..32usize) {
            let pw = randombytes(i);
            let pwh = pwhash(&pw, OPSLIMIT_INTERACTIVE, MEMLIMIT_INTERACTIVE).unwrap();
            assert!(pwhash_verify(&pwh, &pw));
        }
    }

    #[test]
    fn test_pwhash_verify_tamper() {
        use randombytes::randombytes;
        for i in (0..16usize) {
            let mut pw = randombytes(i);
            let pwh = pwhash(&pw, OPSLIMIT_INTERACTIVE, MEMLIMIT_INTERACTIVE).unwrap();
            for j in (0..pw.len()) {
                pw[j] ^= 0x20;
                assert!(!pwhash_verify(&pwh, &pw));
                pw[j] ^= 0x20;
            }
        }
    }
}
