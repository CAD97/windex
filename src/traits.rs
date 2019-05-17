use {
    crate::{
        proof::{NonEmpty, Unknown},
        Container, Index, IndexError,
    },
    core::{cmp, ops},
};

/// Types that can back a trusted container: it can have indices and ranges
/// that are trusted to be in bounds. See also [`TrustedItem`], [`TrustedUnit`].
pub unsafe trait TrustedContainer {
    /// The item type of this container.
    type Item: ?Sized + TrustedItem<Self>;
    /// The slice type of this container.
    type Slice: ?Sized;

    /// The length of the container in base item units.
    fn unit_len(&self) -> u32;

    unsafe fn get_unchecked(&self, i: usize) -> &Self::Item;
    unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice;
}

pub unsafe trait TrustedContainerMut: TrustedContainer {
    unsafe fn get_unchecked_mut(&mut self, i: usize) -> &Self::Item;
    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<usize>) -> &Self::Slice;
}

// FUTURE: feature(arbitrary_self_types) for `this: Index<'id, _>`?
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
    fn vet<'id>(
        idx: u32,
        container: &Container<'id, Array>,
    ) -> Result<Index<'id, Unknown>, IndexError> {
        let len = container.end().untrusted();
        match idx {
            i if i == len => unsafe { Ok(Index::new(idx)) },
            i if i < len => unsafe {
                Self::vet_inbounds(idx, container)
                    .map(Index::erased)
                    .ok_or(IndexError::Invalid)
            },
            _ => Err(IndexError::OutOfBounds),
        }
    }

    /// Vet an untrusted index for being on item boundaries.
    ///
    /// This assumes a proof that the raw index is inbounds. If you
    /// don't have a proof, use [`vet`][`TrustedItem::vet`] instead.
    unsafe fn vet_inbounds<'id>(
        idx: u32,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, NonEmpty>>;

    /// The largest trusted index less than or equal to this untrusted index.
    fn align<'id>(idx: u32, container: &Container<'id, Array>) -> Index<'id, Unknown> {
        let len = container.end().untrusted();
        if idx >= len {
            unsafe { Index::new(len) }
        } else {
            unsafe { Self::align_inbounds(idx, container).erased() }
        }
    }

    /// The largest trusted index less than or equal to this untrusted index.
    ///
    /// This assumes a proof that the raw index is inbounds. If you
    /// don't have a proof, use [`align`][`TrustedItem::align`] instead.
    unsafe fn align_inbounds<'id>(
        idx: u32,
        container: &Container<'id, Array>,
    ) -> Index<'id, NonEmpty>;

    /// Increment an index to the next item, potentially resulting in an index
    /// that is one-past-the-end of the container.
    fn after<'id>(
        this: Index<'id, NonEmpty>,
        container: &Container<'id, Array>,
    ) -> Index<'id, Unknown>;

    /// Advance an index to the next item, if a next item exists.
    fn advance<'id>(
        this: Index<'id, NonEmpty>,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, NonEmpty>> {
        let after = Self::after(this, container);
        // TODO: This should be `Index::nonempty_in`
        if after < container.end() {
            unsafe { Some(Index::new_nonempty(after.untrusted())) }
        } else {
            None
        }
    }

    /// Decrement an index to the previous item, if a previous item exists.
    fn before<'id, P>(
        this: Index<'id, P>,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, NonEmpty>> {
        if this.untrusted() > 0 {
            let aligned = Self::align(this.untrusted() - 1, container);
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
    fn unit_vet<'id>(
        idx: u32,
        container: &Container<'id, Array>,
    ) -> Result<Index<'id, Unknown>, IndexError> {
        if idx <= container.unit_len() {
            Ok(unsafe { Index::new(idx) })
        } else {
            Err(IndexError::OutOfBounds)
        }
    }

    /// Vet an untrusted index for being on item boundaries.
    ///
    /// This assumes a proof that the raw index is inbounds. If you
    /// don't have a proof, use [`vet`][`TrustedUnit::vet`] instead.
    unsafe fn unit_vet_inbounds<'id>(
        idx: u32,
        _: &Container<'id, Array>,
    ) -> Option<Index<'id, NonEmpty>> {
        Some(Index::new_nonempty(idx))
    }

    /// The largest trusted index less than or equal to this untrusted index.
    fn unit_align<'id>(idx: u32, container: &Container<'id, Array>) -> Index<'id, Unknown> {
        unsafe { Index::new(cmp::min(idx, container.end().untrusted())) }
    }

    /// The largest trusted index less than or equal to this untrusted index.
    ///
    /// This assumes a proof that the raw index is inbounds. If you
    /// don't have a proof, use [`align`][`TrustedItem::align`] instead.
    #[allow(clippy::needless_lifetimes)]
    unsafe fn unit_align_inbounds<'id>(idx: u32) -> Index<'id, NonEmpty> {
        Index::new(idx).trusted()
    }

    /// Increment an index to the next item, potentially resulting in an index
    /// that is one-past-the-end of the container.
    #[allow(clippy::needless_lifetimes)]
    fn unit_after<'id>(this: Index<'id, NonEmpty>) -> Index<'id, Unknown> {
        unsafe { Index::new(this.untrusted() + 1) }
    }

    /// Advance an index to the next item, if a next item exists.
    fn unit_advance<'id>(
        this: Index<'id, NonEmpty>,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, NonEmpty>> {
        if this.untrusted() < container.unit_len() {
            unsafe { Some(Index::new_nonempty(this.untrusted() + 1)) }
        } else {
            None
        }
    }

    /// Decrement an index to the previous item, if a previous item exists.
    #[allow(clippy::needless_lifetimes)]
    fn unit_before<'id, P>(this: Index<'id, P>) -> Option<Index<'id, NonEmpty>> {
        if this.untrusted() > 0 {
            unsafe { Some(Index::new_nonempty(this.untrusted() - 1)) }
        } else {
            None
        }
    }
}

macro_rules! __trusted_item_forwarding {
    ({$($bounds:tt)*} $Array:ty => $T:ty) => {
        unsafe impl<$($bounds)*> TrustedItem<$Array> for $T {
            type Unit = $T;

            fn vet<'id>(
                idx: u32,
                container: &Container<'id, $Array>,
            ) -> Result<Index<'id, Unknown>, IndexError> {
                <$T>::unit_vet(idx, container)
            }

            unsafe fn vet_inbounds<'id>(
                idx: u32,
                container: &Container<'id, $Array>,
            ) -> Option<Index<'id, NonEmpty>> {
                <$T>::unit_vet_inbounds(idx, container)
            }

            fn align<'id>(idx: u32, container: &Container<'id, $Array>) -> Index<'id, Unknown> {
                <$T>::unit_align(idx, container)
            }

            unsafe fn align_inbounds<'id>(idx: u32, _: &Container<'id, $Array>) -> Index<'id, NonEmpty> {
                <$T>::unit_align_inbounds(idx)
            }

            fn after<'id>(
                this: Index<'id, NonEmpty>,
                _: &Container<'id, $Array>,
            ) -> Index<'id, Unknown> {
                <$T>::unit_after(this)
            }

            fn advance<'id>(
                this: Index<'id, NonEmpty>,
                container: &Container<'id, $Array>,
            ) -> Option<Index<'id, NonEmpty>> {
                <$T>::unit_advance(this, container)
            }

            fn before<'id, P>(
                this: Index<'id, P>,
                _: &Container<'id, $Array>,
            ) -> Option<Index<'id, NonEmpty>> {
                <$T>::unit_before(this)
            }
        }
    };
}
pub(crate) use __trusted_item_forwarding as trusted_item_forwarding;
