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

//! Polynomial utilites (currently only evaluation).

use crate::fp4::Fp4;

/// Evaluate a polynomial whose coeffients are in the extension field at a
/// point.
pub fn poly_eval(coeffs: &[Fp4], x: Fp4) -> Fp4 {
    let mut mul = Fp4::one();
    let mut tot = Fp4::zero();
    for i in 0..coeffs.len() {
        tot += coeffs[i] * mul;
        mul *= x;
    }
    tot
}
