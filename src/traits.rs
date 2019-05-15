use {
    crate::{
        proof::{NonEmpty, Unknown},
        Container, Index, IndexError,
    },
    core::{cmp, convert::TryFrom, fmt, hash::Hash, ops},
};

/// Unsigned integers able to be used as a trusted index.
pub unsafe trait Idx: Copy + Ord + Hash
where
    Self: fmt::Debug,
{
    //noinspection RsSelfConvention
    fn as_usize(self) -> usize;
    fn from_usize(i: usize) -> Option<Self>;
    fn zero() -> Self;
    fn add(self, increment: usize) -> Self;
    fn sub(self, decrement: usize) -> Self;
}

#[allow(non_snake_case)]
macro_rules! impl_Idx {
    ($($ty:ty),*$(,)?) => {
        $(unsafe impl Idx for $ty {
            fn as_usize(self) -> usize { self as usize } // we need u32 support, 64 KiB isn't enough
            fn from_usize(i: usize) -> Option<Self> { Self::try_from(i).ok() }
            fn zero() -> Self { 0 }
            fn add(self, increment: usize) -> Self {
                // FUTURE: See if this is always sound in the face of wrapping arithmetic
                Self::from_usize(increment).and_then(|i| self.checked_add(i)).expect("increment")
            }
            fn sub(self, decrement: usize) -> Self {
                self - Self::from_usize(decrement).expect("increment")
            }
        })*
    };
}

impl_Idx! {
    u8,
    u16,
    u32,
    usize,
}

/// Types that can back a trusted container: it can have indices and ranges
/// that are trusted to be in bounds. See also [`TrustedItem`], [`TrustedUnit`].
pub unsafe trait TrustedContainer {
    /// The item type of this container.
    type Item: ?Sized + TrustedItem<Self>;
    /// The slice type of this container.
    type Slice: ?Sized;

    /// The length of the container in base item units.
    fn unit_len(&self) -> usize;

    unsafe fn get_unchecked(&self, i: usize) -> &Self::Item;
    unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice;
}

pub unsafe trait TrustedContainerMut: TrustedContainer {
    unsafe fn get_unchecked_mut(&mut self, i: usize) -> &Self::Item;
    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<usize>) -> &Self::Slice;
}

// FUTURE: feature(arbitrary_self_types) for `this: Index<'id, I, _>`?
/// An item within a [`TrustedContainer`].
///
/// Note that raw indices are _unit_ indices, not item indices. One item (e.g.
/// a character) can be made up of multiple units (e.g. bytes).
pub unsafe trait TrustedItem<Array: ?Sized>
where
    Array: TrustedContainer<Item = Self>,
{
    /// The base representational unit type.
    type Unit;

    /// Vet an untrusted index for being on item boundaries.
    ///
    /// The index for one-past-the-end of the container is a valid index. This
    /// method conveys no emptiness proof (use [`Container::vet`] for that).
    fn vet<'id, I: Idx>(
        idx: I,
        container: &Container<'id, Array>,
    ) -> Result<Index<'id, I, Unknown>, IndexError>;

    /// The largest trusted index less than or equal to this untrusted index.
    fn align<'id, I: Idx>(idx: I, container: &Container<'id, Array>) -> Index<'id, I, Unknown>;

    /// Increment an index to the next item, potentially resulting in an index
    /// that is one-past-the-end of the container.
    fn after<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, Array>,
    ) -> Index<'id, I, Unknown>;

    /// Advance an index to the next item, if a next item exists.
    fn advance<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        let after = Self::after(this, container);
        // TODO: This should be `Index::nonempty_in`
        if after < container.end() {
            unsafe { Some(Index::new_nonempty(after.untrusted())) }
        } else {
            None
        }
    }

    /// Decrement an index to the previous item, if a previous item exists.
    fn before<'id, I: Idx, P>(
        this: Index<'id, I, P>,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        if this.untrusted() > I::zero() {
            let aligned = Self::align(this.untrusted().sub(1), container);
            unsafe { Some(Index::new_nonempty(aligned.untrusted())) }
        } else {
            None
        }
    }
}

//noinspection RsNeedlessLifetimes [intellij-rust/intellij-rust#3844]
/// A [`TrustedItem`] where the item is the base unit. Thus, manipulating
/// indices and ranges of the container is as simple as regular arithmetic.
pub unsafe trait TrustedUnit<Array: ?Sized>:
    TrustedItem<Array, Unit = Self> + Sized
where
    Array: TrustedContainer<Item = Self, Slice = [Self]>,
{
    /// Vet an untrusted index for being on item boundaries.
    ///
    /// The index for one-past-the-end of the container is a valid index. This
    /// method conveys no emptiness proof (use [`Container::vet`] for that).
    fn unit_vet<'id, I: Idx>(
        idx: I,
        container: &Container<'id, Array>,
    ) -> Result<Index<'id, I, Unknown>, IndexError> {
        if idx.as_usize() <= container.unit_len() {
            Ok(unsafe { Index::new(idx) })
        } else {
            Err(IndexError::OutOfBounds)
        }
    }

    /// The largest trusted index less than or equal to this untrusted index.
    fn unit_align<'id, I: Idx>(
        idx: I,
        container: &Container<'id, Array>,
    ) -> Index<'id, I, Unknown> {
        unsafe { Index::new(cmp::min(idx, container.end().untrusted())) }
    }

    /// Increment an index to the next item, potentially resulting in an index
    /// that is one-past-the-end of the container.
    #[allow(clippy::needless_lifetimes)]
    fn unit_after<'id, I: Idx>(this: Index<'id, I, NonEmpty>) -> Index<'id, I, Unknown> {
        unsafe { Index::new(this.untrusted().add(1)) }
    }

    /// Advance an index to the next item, if a next item exists.
    fn unit_advance<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        if this.untrusted().as_usize() < container.unit_len() {
            unsafe { Some(Index::new_nonempty(this.untrusted().add(1))) }
        } else {
            None
        }
    }

    /// Decrement an index to the previous item, if a previous item exists.
    #[allow(clippy::needless_lifetimes)]
    fn unit_before<'id, I: Idx, P>(this: Index<'id, I, P>) -> Option<Index<'id, I, NonEmpty>> {
        if this.untrusted() > I::zero() {
            unsafe { Some(Index::new_nonempty(this.untrusted().sub(1))) }
        } else {
            None
        }
    }
}

macro_rules! __trusted_item_forwarding {
    ({$($bounds:tt)*} $Array:ty => $T:ty) => {
        unsafe impl<$($bounds)*> TrustedItem<$Array> for $T {
            type Unit = $T;

            fn vet<'id, I: Idx>(
                idx: I,
                container: &Container<'id, $Array>,
            ) -> Result<Index<'id, I, Unknown>, IndexError> {
                <$T>::unit_vet(idx, container)
            }

            fn align<'id, I: Idx>(idx: I, container: &Container<'id, $Array>) -> Index<'id, I, Unknown> {
                <$T>::unit_align(idx, container)
            }

            fn after<'id, I: Idx>(
                this: Index<'id, I, NonEmpty>,
                _: &Container<'id, $Array>,
            ) -> Index<'id, I, Unknown> {
                <$T>::unit_after(this)
            }

            fn advance<'id, I: Idx>(
                this: Index<'id, I, NonEmpty>,
                container: &Container<'id, $Array>,
            ) -> Option<Index<'id, I, NonEmpty>> {
                <$T>::unit_advance(this, container)
            }

            fn before<'id, I: Idx, P>(
                this: Index<'id, I, P>,
                _: &Container<'id, $Array>,
            ) -> Option<Index<'id, I, NonEmpty>> {
                <$T>::unit_before(this)
            }
        }
    };
}
pub(crate) use __trusted_item_forwarding as trusted_item_forwarding;
