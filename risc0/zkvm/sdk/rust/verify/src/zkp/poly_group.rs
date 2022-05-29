use risc0_zkp_core::fp::Fp;

use crate::zkp::{
    hal::{self, BoxedSlice},
    log2_ceil,
    merkle::MerkleTreeProver,
    INV_RATE, QUERIES,
};

/// A PolyGroup represents a group of polynomials, all of the same maximum degree, as well as the
/// evaluation of those polynomials over some domain that is larger than that degree by some invRate.
/// Additionally, it includes a dense Merkle tree, where each entry is a single point of the domain,
/// and the leaf hash is a simple linear hash of all of the values at that point.  That is, if we
/// have 100 polynomials evaluated on 2^16 points, the merkle tree has 2^16 entries, each being a
/// hash of 100 values.  The size of the domain is always a power of 2 so that we can use NTTs.
///
/// The primary purpose of the PolyGroup is for use in the DEEP-ALI protocol, which basically needs
/// 4 methods during proof generation, specifically we need to:
/// 1) Resolve queries (i.e. make MerkleColProofs)
/// 2) Do evaluation of the polynomials at 'randomly' chosen points
/// 3) Mix the polynomials via a random set of linear coefficients
/// 4) Access the raw values in the evaluation domain to 'evaluate' the constraint polynomial
///
/// The poly group holds 3 buffers:
/// 1) The per-polynomial coefficients, used for evaluation + mixing
/// 2) The points evaluated on the domain in question (for the 'col' part of merkle proofs)
/// 3) The Merkle tree itself.
///
/// PolyGroups are constructed from two basic sources: steps of a computations, and a single higher
/// degree polynomial that has been split into lower degree parts.  In the case of computations, the
/// resulting steps must be padded (possibly with randomized data), which is presumed to be done by
/// the caller. The constructor additionally 'shifts' the polynomial so that f(x) -> f(3*x), which
/// means that the normal NTT evaluation domain does not reveal anything about the original
/// datapoints (i.e. is zero knowledge) so long as the number of queries is less than the randomized
/// padding.
pub struct PolyGroup {
    pub coeffs: BoxedSlice<Fp>,
    pub count: usize,
    size: usize,
    domain: usize,
    pub evaluated: BoxedSlice<Fp>,
    pub merkle: MerkleTreeProver,
}

impl PolyGroup {
    pub fn new(coeffs: BoxedSlice<Fp>, count: usize, size: usize) -> Self {
        assert_eq!(coeffs.size(), count * size);
        let domain = size * INV_RATE;
        let mut evaluated = hal::alloc(count * domain);
        hal::batch_expand(&mut *evaluated, &*coeffs, count);
        hal::batch_evaluate_ntt(&*evaluated, count, log2_ceil(INV_RATE));
        hal::batch_bit_reverse(&*coeffs, count);
        let merkle = MerkleTreeProver::new(&evaluated, domain, count, QUERIES);
        let size = coeffs.size() / count;
        PolyGroup {
            coeffs,
            count,
            size,
            domain,
            evaluated,
            merkle,
        }
    }
}
