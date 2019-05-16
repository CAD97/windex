#[cfg(feature = "doc")]
use crate::scope;
use {
    crate::{
        Container, Range,
        proof::{Id, NonEmpty, Unknown},
        traits::{Idx, TrustedContainer},
    },
    core::{
        cmp, fmt,
        hash::{self, Hash},
        marker::PhantomData,
    },
};

/// A branded index.
///
/// `Index<'id>` only indexes the container instantiated with the exact same
/// lifetime for the parameter `'id` created by the [`scope`] function.
///
/// The type parameter `Emptiness` determines if the index is followable.
/// A `NonEmpty` index points to a valid element. An `Unknown` index is
/// unknown, or it points to an edge index (one past the end).
pub struct Index<'id, I: Idx = u32, Emptiness = NonEmpty> {
    #[allow(unused)]
    id: Id<'id>,
    idx: I,
    phantom: PhantomData<Emptiness>,
}

// ~~~ Private Helpers ~~~ //

impl<'id, I: Idx> Index<'id, I, Unknown> {
    pub(crate) unsafe fn new(idx: I) -> Self {
        Index::new_any(idx)
    }
}

impl<'id, I: Idx> Index<'id, I, NonEmpty> {
    pub(crate) unsafe fn new_nonempty(idx: I) -> Self {
        Index::new_any(idx)
    }
}

impl<'id, I: Idx, Emptiness> Index<'id, I, Emptiness> {
    pub(crate) unsafe fn new_any(idx: I) -> Self {
        Index {
            id: Id::default(),
            idx,
            phantom: PhantomData,
        }
    }

    pub(crate) unsafe fn trusted(&self) -> Index<'id, I, NonEmpty> {
        Index::new_nonempty(self.untrusted())
    }
}

// ~~~ Discarding Proofs ~~~ //

impl<'id, I: Idx, Emptiness> Index<'id, I, Emptiness> {
    /// This index without the branding.
    pub fn untrusted(&self) -> I {
        self.idx
    }

    /// This index without the emptiness proof.
    pub fn erased(&self) -> Index<'id, I, Unknown> {
        unsafe { Index::new(self.idx) }
    }
}

// ~~~ Gaining Proofs ~~~ //

impl<'id, I: Idx, Emptiness> Index<'id, I, Emptiness> {
    /// Try to create a proof that this index is nonempty.
    pub fn nonempty_in<Array: ?Sized + TrustedContainer>(
        &self,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        if *self < container.end() {
            unsafe { Some(self.trusted()) }
        } else {
            None
        }
    }

    /// Try to create a proof that this index is within a range.
    pub fn in_range<Q>(&self, range: Range<'id, I, Q>) -> Option<Index<'id, I, NonEmpty>> {
        if *self >= range.start() && *self < range.end() {
            unsafe { Some(self.trusted()) }
        } else {
            None
        }
    }
}

// ~~~ Derive traits but without unneeded bounds ~~~ //

impl<'id, I: Idx, Emptiness> fmt::Debug for Index<'id, I, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Index<'id>").field(&self.idx).finish()
    }
}

impl<'id, I: Idx, Emptiness> Copy for Index<'id, I, Emptiness> {}
impl<'id, I: Idx, Emptiness> Clone for Index<'id, I, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, I: Idx, Emptiness> Ord for Index<'id, I, Emptiness> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl<'id, I: Idx, P, Q> PartialOrd<Index<'id, I, Q>> for Index<'id, I, P> {
    fn partial_cmp(&self, other: &Index<'id, I, Q>) -> Option<cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl<'id, I: Idx, Emptiness> Eq for Index<'id, I, Emptiness> {}
impl<'id, I: Idx, P, Q> PartialEq<Index<'id, I, Q>> for Index<'id, I, P> {
    fn eq(&self, other: &Index<'id, I, Q>) -> bool {
        self.idx.eq(&other.idx)
    }
}

impl<'id, I: Idx, Emptiness> Hash for Index<'id, I, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
    }
}
