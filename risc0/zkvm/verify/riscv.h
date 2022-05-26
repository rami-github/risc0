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

#pragma once

#include <memory>
#include <array>

#include "risc0/core/util.h"
#include "risc0/zkvm/circuit/constants.h"
#include "risc0/zkp/verify/verify.h"

namespace risc0 {

// Redundant with alias in prove/method_id.h
constexpr size_t kDigestCount = log2Ceil(kMaxCycles / kMinCycles) + 1;
using MethodID = std::array<ShaDigest, kDigestCount>;

TapSetRef getRiscVTaps();
std::unique_ptr<VerifyCircuit> getRiscVVerifyCircuit(const MethodID& id);

} // namespace risc0
