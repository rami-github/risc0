// Copyright 2022 Risc0, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod fri;
mod hal;
mod merkle;
mod poly_group;
mod prove;
pub(crate) mod read_iop;
pub(crate) mod taps;
pub(crate) mod verify;
mod write_iop;

use risc0_zkp_core::fp4::EXT_SIZE;

const MAX_CYCLES_PO2: usize = 20;
const MAX_CYCLES: usize = 1 << MAX_CYCLES_PO2;

/// ~100 bits of conjectured security
pub const QUERIES: usize = 50;

const INV_RATE: usize = 4;
const MAX_DEGREE: usize = INV_RATE + 1;
const FRI_FOLD_PO2: usize = 4;
const FRI_FOLD: usize = 1 << FRI_FOLD_PO2;
const FRI_MIN_DEGREE: usize = 256;

const CHECK_SIZE: usize = INV_RATE * EXT_SIZE;

/// Compute `ceil(log_2(value))`
///
/// Find the smallest value `result` such that `2^result >= value`.
#[inline]
pub fn log2_ceil(value: usize) -> usize {
    let mut result = 0;
    while (1 << result) < value {
        result += 1;
    }
    result
}
