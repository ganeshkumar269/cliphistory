use hex;
use md5::{Digest, Md5};
#[derive(Debug)]
pub struct Clip {
    pub(crate) value: String,
    pub(crate) timestamp: i64,
    pub(crate) hash: String,
}

impl Clip {
    pub fn new(value: String, timestamp: i64) -> Clip {
        let mut hasher = Md5::new();
        hasher.update(value.clone());
        let hash = hex::encode(hasher.finalize());
        // let hash = String::from(hasher.finalize().as_slice());
        Clip {
            value,
            timestamp,
            hash,
        }
    }
}
