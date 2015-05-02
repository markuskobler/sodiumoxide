macro_rules! hash_module (($hash_name:ident, $hashbytes:expr, $blockbytes:expr) => (

use std::ops::{Index, Range, RangeFrom, RangeFull, RangeTo};
use libc::c_ulonglong;

pub const HASHBYTES: usize = $hashbytes;
pub const BLOCKBYTES: usize = $blockbytes;

/// Digest-structure
#[derive(Copy)]
pub struct Digest(pub [u8; HASHBYTES]);

newtype_clone!(Digest);
newtype_impl!(Digest, HASHBYTES);

/// `hash` hashes a message `m`. It returns a hash `h`.
pub fn hash(m: &[u8]) -> Digest {
    unsafe {
        let mut h = [0; HASHBYTES];
        $hash_name(&mut h, m.as_ptr(), m.len() as c_ulonglong);
        Digest(h)
    }
}

#[cfg(feature = "benchmarks")]
#[cfg(test)]
mod bench_m {
    extern crate test;
    use randombytes::randombytes;
    use super::*;

    const BENCH_SIZES: [usize; 14] = [0, 1, 2, 4, 8, 16, 32, 64,
                                      128, 256, 512, 1024, 2048, 4096];

    #[bench]
    fn bench_hash(b: &mut test::Bencher) {
        let ms: Vec<Vec<u8>> = BENCH_SIZES.iter().map(|s| {
            randombytes(*s)
        }).collect();
        b.iter(|| {
            for m in ms.iter() {
                hash(&m);
            }
        });
    }
}

));
