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

use alloc::{vec, vec::Vec};

use risc0_zkp_core::{
    fp::Fp,
    sha::{Digest, Sha},
    to_po2,
};

use crate::zkp::read_iop::ReadIOP;

/// The parameters of a merkle tree of prime field elements, including:
/// row_size - the number of leaves in the tree
/// col_size - the number of field elements associated with each leaf
/// queries - the number of queries that will be made to this merkle tree
/// layers - the number of levels on the merkle tree
/// top_layer - the index of the layer above which we check hashes only once
/// top_size - the number of hashes in the top layer
pub struct MerkleTreeParams {
    pub row_size: usize,
    pub col_size: usize,
    pub queries: usize,
    pub layers: usize,
    pub top_layer: usize,
    pub top_size: usize,
}

impl MerkleTreeParams {
    /// Returns the parameters of the Merkle tree, given the row and column size
    /// and the number of queries to be made.
    pub fn new(row_size: usize, col_size: usize, queries: usize) -> Self {
        // The number of layers is the logarithm base 2 of the row_size.
        let layers: usize = to_po2(row_size);
        assert!(1 << layers == row_size);
        // The "top" layer is a layer above which we verify all Merkle data only once at
        // the beginning. Later, we only verify merkle branches from the leaf up
        // to this top layer. This allows us to avoid checking hashes in this
        // part of the tree multiple times. We choose the top layer to be the
        // one with size at most equal to queries.
        let mut top_layer = 0;
        for i in 1..layers {
            if (1 << i) > queries {
                break;
            }
            top_layer = i;
        }
        let top_size = 1 << top_layer;
        MerkleTreeParams {
            row_size,
            col_size,
            queries,
            layers,
            top_layer,
            top_size,
        }
    }
}

/// A struct against which we verify merkle branches, consisting of the
/// parameters of the Merkle tree and top - the vector of hash values in the top
/// row of the tree, above which we verify only once.
pub struct MerkleTreeVerifier {
    params: MerkleTreeParams,
    top: Vec<Digest>,
}

impl MerkleTreeVerifier {
    /// Constructs a new MerkleTreeVerifier by making the params, and then
    /// computing the root hashes from the top level hashes.
    pub fn new<S: Sha>(
        iop: &mut ReadIOP<S>,
        row_size: usize,
        col_size: usize,
        queries: usize,
    ) -> Self {
        let sha = iop.get_sha().clone();
        let params = MerkleTreeParams::new(row_size, col_size, queries);
        // Initialize a vector to hold the digests.
        // Vector is twice as long as the "top" row - the children of the entry at index
        // i are stored at 2*i and 2*i+1.
        let mut top = vec![Digest::default(); params.top_size * 2];
        // Fill top vector with digests from IOP.
        iop.read_digests(&mut top[params.top_size..]);
        // Populate hashes up to the root of the tree.
        for i in (1..params.top_size).rev() {
            top[i] = *sha.hash_pair(&top[2 * i], &top[2 * i + 1]);
        }
        // Commit to root (index 1).
        iop.commit(&top[1]);
        return MerkleTreeVerifier { params, top };
    }

    /// Returns the root hash of the tree.
    pub fn root(&self) -> &Digest {
        return &self.top[1];
    }

    /// Verifies a branch provided by an IOP.
    pub fn verify<S: Sha>(&self, iop: &mut ReadIOP<S>, mut idx: usize) -> Vec<Fp> {
        let sha = iop.get_sha().clone();
        let col_size = self.params.col_size;
        let row_size = self.params.row_size;
        assert!(idx < row_size);
        // Initialize a vector to hold field elements.
        let mut out: Vec<Fp> = vec![Fp::new(0); col_size];
        // Read out field elements from IOP.
        iop.read_fps(&mut out);
        // Get the hash at the leaf of the tree by hashing these field elements.
        let mut cur: Digest = *sha.hash_fps(&out);
        // Shift idx to start of the row
        idx += row_size;
        while idx >= 2 * self.params.top_size {
            // low_bit determines whether hash cur at idx is the left (0) or right (1)
            // child.
            let low_bit = idx % 2;
            // Retrieve the other parent from the IOP.
            let mut other = Digest::default();
            iop.read_digests(core::slice::from_mut(&mut other));
            // Now ascend to the parent index, and compute the hash there.
            idx /= 2;
            if low_bit == 1 {
                cur = *sha.hash_pair(&other, &cur);
            } else {
                cur = *sha.hash_pair(&cur, &other);
            }
        }
        // Once we reduce to an index for which we have the hash, check that it's
        // correct.
        assert!(self.top[idx] == cur);
        out
    }
}
