use std::num::Wrapping;

use serde::{Deserialize, Serialize};

/// To advance the `rng1` value, it is multiplied by this number.
const RNG1_ADVANCE: u32 = 0x19660d;

/// A 256-byte table containing a permutation of the integer values from 0-255.
/// An `rng2` value of `v` is interpreted as the random byte `RNG2_VALUES[v % 256]`.
/// offset: 0x60480
const RNG2_VALUES: &'static [u8; 256] = include_bytes!("bin/rng2_values.bin");

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RngState {
    /// RNG1 value. Initial value is 0x10e1 (4321 in decimal).
    /// offset: 0x7bc44
    pub rng1: Wrapping<u32>,

    /// The number of times the `rng1` value has been advanced.
    /// offset: 0x153140
    rng1_calls: u32,

    /// RNG2 value. Initial value is 0.
    /// offset: 0x10eb2c
    pub rng2: u32,
}

impl Default for RngState {
    fn default() -> Self {
        // The default RNG values are set in the function at offset 0x5d10.
        Self {
            rng1: Wrapping(4321),
            rng1_calls: 0,
            rng2: 0,
        }
    }
}

impl RngState {
    pub fn get_rng1(&mut self) -> u32 {
        // save original rng1 state to return
        let result = self.rng1;

        // update rng1 state
        self.rng1 = self.rng1 * Wrapping(RNG1_ADVANCE);
        self.rng1_calls += 1;

        // return original rng2 state
        result.0
    }

    pub fn get_rng2(&mut self) -> u8 {
        // save original rng2 state to return
        let current = self.rng2;
        let result = RNG2_VALUES[(current & 255) as usize];

        // update rng2 state
        self.rng2 += 1;

        // return original rng2 state
        result
    }
}
