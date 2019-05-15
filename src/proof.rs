use crate::traits::Idx;
use {
    crate::{Index, Range},
    core::{fmt, marker::PhantomData},
};

/// `Id<'id>` is _invariant_ w.r.t. `'id`.
///
/// This means that the inference engine is not allowed to
/// grow or shrink `'id` to unify with any other lifetime.
#[derive(Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub(crate) struct Id<'id> {
    id: PhantomData<&'id mut &'id ()>,
}

unsafe impl<'id> Sync for Id<'id> {}
unsafe impl<'id> Send for Id<'id> {}

impl<'id> fmt::Debug for Id<'id> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Id<'id>").finish()
    }
}

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

pub trait Provable {
    type Proof;
    type WithoutProof: Provable<Proof = Unknown>;

    /// Return a copy of self with the proof parameter set to `Unknown`.
    fn no_proof(self) -> Self::WithoutProof;
}

impl<'id, I: Idx, Emptiness> Provable for Index<'id, I, Emptiness> {
    type Proof = Emptiness;
    type WithoutProof = Index<'id, I, Unknown>;

    fn no_proof(self) -> Self::WithoutProof {
        unsafe { Index::new(self.untrusted()) }
    }
}

impl<'id, I: Idx, Emptiness> Provable for Range<'id, I, Emptiness> {
    type Proof = Emptiness;
    type WithoutProof = Range<'id, I, Unknown>;

    fn no_proof(self) -> Self::WithoutProof {
        unsafe { Range::new(self.start().untrusted(), self.end().untrusted()) }
    }
}
