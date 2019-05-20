use {
    crate::{particle::perfect, proof::*},
    core::{
        cmp,
        fmt::{self, Debug},
        hash::{self, Hash},
        marker::PhantomData,
    },
};

pub struct Index<'id, Emptiness = NonEmpty> {
    #[allow(unused)]
    id: Id<'id>,
    ix: u32,
    phantom: PhantomData<Emptiness>,
}

/// Constructors
impl<'id, Emptiness> Index<'id, Emptiness> {
    pub(crate) unsafe fn new(ix: u32) -> Self {
        Index {
            id: Id::default(),
            ix,
            phantom: PhantomData,
        }
    }
}

/// Downgrade
impl<'id, Emptiness> Index<'id, Emptiness> {
    /// This index without the brand.
    pub fn untrusted(self) -> u32 {
        self.ix
    }

    /// This index without the emptiness proof.
    pub fn erased(self) -> Index<'id, Unknown> {
        unsafe { Index::new(self.ix) }
    }
}

/// Manipulation
impl<'id> Index<'id, NonEmpty> {
    /// The (simple) index directly after this one.
    pub fn after(self) -> Index<'id, Unknown> {
        unsafe { Index::new(self.ix + 1) }
    }
}

// ~~~ Standard traits ~~~ //

impl<'id, Emptiness> From<perfect::Index<'id, Emptiness>> for Index<'id, Emptiness> {
    fn from(index: perfect::Index<'id, Emptiness>) -> Self {
        index.simple()
    }
}

impl<'id, Emptiness> Copy for Index<'id, Emptiness> {}

impl<'id, Emptiness> Clone for Index<'id, Emptiness> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'id, Emptiness> Debug for Index<'id, Emptiness> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("simple::Index<'id>").finish()
    }
}

impl<'id> Default for Index<'id, Unknown> {
    fn default() -> Self {
        unsafe { Self::new(0) }
    }
}

impl<'id, Emptiness> Ord for Index<'id, Emptiness> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.ix.cmp(&other.ix)
    }
}

impl<'id, 'jd, Emptiness, P> PartialOrd<Index<'jd, P>> for Index<'id, Emptiness> {
    fn partial_cmp(&self, other: &Index<'jd, P>) -> Option<cmp::Ordering> {
        self.ix.partial_cmp(&other.ix)
    }
}

impl<'id, 'jd, Emptiness, P> PartialOrd<perfect::Index<'jd, P>> for Index<'id, Emptiness> {
    fn partial_cmp(&self, other: &perfect::Index<'jd, P>) -> Option<cmp::Ordering> {
        self.ix.partial_cmp(&other.simple().ix)
    }
}

impl<'id, Emptiness> Eq for Index<'id, Emptiness> {}

impl<'id, 'jd, Emptiness, P> PartialEq<Index<'jd, P>> for Index<'id, Emptiness> {
    fn eq(&self, other: &Index<'jd, P>) -> bool {
        self.ix.eq(&other.ix)
    }
}

impl<'id, 'jd, Emptiness, P> PartialEq<perfect::Index<'jd, P>> for Index<'id, Emptiness> {
    fn eq(&self, other: &perfect::Index<'jd, P>) -> bool {
        self.ix.eq(&other.simple().ix)
    }
}

impl<'id, Emptiness> Hash for Index<'id, Emptiness> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.ix.hash(state)
    }
}
