/// Length marker for range/index known to not be empty.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum NonEmpty {}

/// Length marker for range/index of unknown length (may be empty).
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Unknown {}

/// Represents the combination of two proofs `P` and `Q` by a new type `Sum`.
pub trait ProofAdd {
    type Sum;
}

impl<Q> ProofAdd for (NonEmpty, Q) {
    type Sum = NonEmpty;
}
impl<Q> ProofAdd for (Unknown, Q) {
    type Sum = Q;
}
