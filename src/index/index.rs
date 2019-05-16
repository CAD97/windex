#[cfg(feature = "doc")]
use crate::scope;
use {
    crate::{
        proof::{Id, NonEmpty, Unknown},
        traits::TrustedContainer,
        Container, Range,
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
//
//  # Safety
//
//  The raw index must be 0 <= idx <= len for the branded container.
//  When Emptiness = NonEmpty, 0 <= idx < len.
pub struct Index<'id, Emptiness = NonEmpty> {
    #[allow(unused)]
    id: Id<'id>,
    idx: u32,
    phantom: PhantomData<Emptiness>,
}

// ~~~ Private Helpers ~~~ //

impl<'id> Index<'id, Unknown> {
    pub(crate) unsafe fn new(idx: u32) -> Self {
        Index::new_any(idx)
    }
}

impl<'id> Index<'id, NonEmpty> {
    pub(crate) unsafe fn new_nonempty(idx: u32) -> Self {
        Index::new_any(idx)
    }
}

impl<'id, Emptiness> Index<'id, Emptiness> {
    pub(crate) unsafe fn new_any(idx: u32) -> Self {
        Index {
            id: Id::default(),
            idx,
            phantom: PhantomData,
        }
    }

    pub(crate) unsafe fn trusted(self) -> Index<'id, NonEmpty> {
        Index::new_nonempty(self.untrusted())
    }
}

// ~~~ Discarding Proofs ~~~ //

impl<'id, Emptiness> Index<'id, Emptiness> {
    /// This index without the branding.
    pub fn untrusted(self) -> u32 {
        self.idx
    }

    /// This index without the emptiness proof.
    pub fn erased(self) -> Index<'id, Unknown> {
        unsafe { Index::new(self.idx) }
    }
}

// ~~~ Gaining Proofs ~~~ //

impl<'id, Emptiness> Index<'id, Emptiness> {
    /// Try to create a proof that this index is nonempty.
    pub fn nonempty_in<Array: ?Sized + TrustedContainer>(
        self,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, NonEmpty>> {
        if self < container.end() {
            unsafe { Some(self.trusted()) }
        } else {
            None
        }
    }

    /// Try to create a proof that this index is within a range.
    pub fn in_range<Q>(self, range: Range<'id, Q>) -> Option<Index<'id, NonEmpty>> {
        if self >= range.start() && self < range.end() {
            unsafe { Some(self.trusted()) }
        } else {
            None
        }
    }
}

// ~~~ Derive traits but without unneeded bounds ~~~ //

impl<'id, Emptiness> fmt::Debug for Index<'id, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Index<'id>").field(&self.idx).finish()
    }
}

impl<'id, Emptiness> Copy for Index<'id, Emptiness> {}

impl<'id, Emptiness> Clone for Index<'id, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, Emptiness> Ord for Index<'id, Emptiness> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl<'id, P, Q> PartialOrd<Index<'id, Q>> for Index<'id, P> {
    fn partial_cmp(&self, other: &Index<'id, Q>) -> Option<cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl<'id, Emptiness> Eq for Index<'id, Emptiness> {}

impl<'id, P, Q> PartialEq<Index<'id, Q>> for Index<'id, P> {
    fn eq(&self, other: &Index<'id, Q>) -> bool {
        self.idx.eq(&other.idx)
    }
}

impl<'id, Emptiness> Hash for Index<'id, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
    }
}
