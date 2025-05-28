use std::time::{SystemTime, UNIX_EPOCH};
use hex;
use md5::{Digest, Md5};
#[derive(Debug)]
pub struct Clip {
    pub(crate) value: String,
    pub(crate) timestamp: i64,
    pub(crate) hash: String,
    pub(crate) source: String,
}

impl Clip {
    pub fn new(value: String, source: String) -> Clip {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mut hasher = Md5::new();
        hasher.update(value.clone());
        let hash = hex::encode(hasher.finalize());
        Clip {
            value,
            timestamp: millis as i64,
            hash,
            source
        }
    }
}
