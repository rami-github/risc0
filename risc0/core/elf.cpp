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

#include "risc0/core/elf.h"

#include <fstream>
#include <set>
#include <sstream>
#include <vector>

namespace risc0 {

namespace {

struct ElfHeader {
  uint8_t ei_magic[4];
  uint8_t ei_class;
  uint8_t ei_data;
  uint8_t ei_version;
  uint8_t ei_pad[9];
  uint16_t e_type;
  uint16_t e_machine;
  uint32_t e_version;
  uint32_t e_entry;
  uint32_t e_phoff;
  uint32_t e_shoff;
  uint32_t e_flags;
  uint16_t e_ehsize;
  uint16_t e_phentsize;
  uint16_t e_phnum;
  uint16_t e_shentsize;
  uint16_t e_shnum;
  uint16_t e_shstrndx;
};

struct ProgHeader {
  uint32_t p_type;
  uint32_t p_offset;
  uint32_t p_vaddr;
  uint32_t p_paddr;
  uint32_t p_filesz;
  uint32_t p_memsz;
  uint32_t p_flags;
  uint32_t p_align;
};

} // namespace

uint32_t loadElf(const std::string& name, uint32_t maxMem, std::map<uint32_t, uint32_t>& memOut) {
  using namespace std;
  ifstream is;
  is.open(name, ios::binary);
  if (is.fail() || is.bad()) {
    std::stringstream ss;
    ss << "Could not load ELF: " << name;
    throw std::runtime_error(ss.str());
  }
  ElfHeader elfHeader;
  vector<ProgHeader> progHeaders;
  // Load the main ELF header
  is.read(reinterpret_cast<char*>(&elfHeader), sizeof(ElfHeader));
  // Validate all the relevant fields
  if (elfHeader.ei_magic[0] != 0x7f || //
      elfHeader.ei_magic[1] != 'E' ||  //
      elfHeader.ei_magic[2] != 'L' ||  //
      elfHeader.ei_magic[3] != 'F') {
    throw runtime_error("Invalid magic number");
  }
  if (elfHeader.ei_class != 1) {
    throw runtime_error("Not a 32 bit elf");
  }
  if (elfHeader.ei_data != 1) {
    throw runtime_error("Not little endian");
  }
  if (elfHeader.ei_version != 1 || elfHeader.e_version != 1) {
    throw runtime_error("Invalid elf version");
  }
  if (elfHeader.e_type != 2) {
    throw runtime_error("Invalid elf type, must be executable");
  }
  if (elfHeader.e_machine != 0xf3) {
    throw runtime_error("Invalid machine type, must be Risc-V");
  }
  if (elfHeader.e_entry >= maxMem || elfHeader.e_entry % 4 != 0) {
    throw runtime_error("Invalid entry point");
  }
  if (elfHeader.e_phnum > 256) {
    throw runtime_error("Too many program headers");
  }
  // Load the program headers
  progHeaders.resize(elfHeader.e_phnum);
  is.seekg(elfHeader.e_phoff, ios::beg);
  is.read(reinterpret_cast<char*>(progHeaders.data()),
          static_cast<streamsize>(progHeaders.size() * sizeof(ProgHeader)));

  // Go to each program header and load it if relevant
  for (size_t i = 0; i < progHeaders.size(); i++) {
    const auto& phdr = progHeaders[i];
    if (phdr.p_type != 1) {
      // Not a 'load' header, skip it!
      continue;
    }
    // Validate program header
    if (phdr.p_memsz > maxMem) {
      throw runtime_error("Program header size too big");
    }
    if (phdr.p_vaddr + phdr.p_memsz >= maxMem) {
      throw runtime_error("Program header loads past end of memory");
    }
    if (phdr.p_vaddr % 4 != 0) {
      throw runtime_error("Program header not aligned");
    }
    if (phdr.p_filesz > phdr.p_memsz) {
      throw runtime_error("Program header may not be larger in file than in memory");
    }
    // Seek to position in file
    is.seekg(phdr.p_offset, ios::beg);
    // Load in memory 4 bytes at a time
    for (uint32_t i = 0; i < phdr.p_memsz; i += 4) {
      if (memOut.count(phdr.p_vaddr + i)) {
        throw runtime_error("Invalid overlapping data");
      }
      if (i >= phdr.p_filesz) {
        // Past the file size, all zeros
        memOut[phdr.p_vaddr + i] = 0;
        continue;
      }
      uint8_t buf[4] = {0, 0, 0, 0};
      size_t toRead = min(phdr.p_filesz - i, uint32_t(4));
      is.read(reinterpret_cast<char*>(buf), static_cast<streamsize>(toRead));
      uint32_t word = uint32_t(buf[0]) | (uint32_t(buf[1]) << 8) | (uint32_t(buf[2]) << 16) |
                      (uint32_t(buf[3]) << 24);
      memOut[phdr.p_vaddr + i] = word;
    }
  }
  return elfHeader.e_entry;
}

} // namespace risc0
