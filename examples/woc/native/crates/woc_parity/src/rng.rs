use crate::{fnv1a_step_u32, FNV_OFFSET};

const ZERO_SEED_REPLACEMENT: u32 = 0x9e37_79b9;
const MULBERRY_INCREMENT: u32 = 0x6d2b_79f5;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 {
                ZERO_SEED_REPLACEMENT
            } else {
                seed
            },
        }
    }

    pub fn state(&self) -> u32 {
        self.state
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_add(MULBERRY_INCREMENT);
        let mut value = self.state;
        value = (value ^ (value >> 15)).wrapping_mul(value | 1);
        value ^= value.wrapping_add((value ^ (value >> 7)).wrapping_mul(value | 61));
        value ^ (value >> 14)
    }

    pub fn next_f64(&mut self) -> f64 {
        f64::from(self.next_u32()) / 4_294_967_296.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawDigest {
    draws: u64,
    hash: u32,
}

impl Default for DrawDigest {
    fn default() -> Self {
        Self {
            draws: 0,
            hash: FNV_OFFSET,
        }
    }
}

impl DrawDigest {
    pub fn observe_u32(&mut self, value: u32) {
        self.draws += 1;
        self.hash = fnv1a_step_u32(self.hash, value);
    }

    pub fn observe_f64(&mut self, value: f64) {
        let draw = (value * 4_294_967_296.0).round() as u64 as u32;
        self.observe_u32(draw);
    }

    pub fn draws(&self) -> u64 {
        self.draws
    }

    pub fn hash(&self) -> u32 {
        self.hash
    }

    pub fn hex(&self) -> String {
        format!("{:08x}", self.hash)
    }
}
