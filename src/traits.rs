use crate::proof::{NonEmpty, Unknown};
use {
    crate::{
        particle::{perfect::*, IndexError},
        Container,
    },
    core::ops,
};

/// Types that can back a trusted container: it can have particles that are
/// trusted to be in bounds. See also [`TrustedItem`], [`TrustedUnit`].
#[allow(clippy::len_without_is_empty)]
pub unsafe trait TrustedContainer {
    /// The item type of this container.
    type Item: ?Sized + TrustedItem<Self>;
    /// The slice type of this container.
    type Slice: ?Sized;

    /// The length of the container in base representation units.
    fn len(&self) -> u32;

    unsafe fn get_unchecked(&self, i: u32) -> &Self::Item;
    unsafe fn slice_unchecked(&self, r: ops::Range<u32>) -> &Self::Slice;
}

pub unsafe trait TrustedContainerMut: TrustedContainer {
    unsafe fn get_unchecked_mut(&mut self, i: u32) -> &mut Self::Item;
    unsafe fn slice_unchecked_mut(&mut self, r: ops::Range<u32>) -> &mut Self::Slice;
}

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
    /// This does not require the index to be nonempty; thus,
    /// the one-past-the-end index is valid for this vetting.
    fn vet<'id>(
        ix: u32,
        container: &Container<'id, Array>,
    ) -> Result<Index<'id, Unknown>, IndexError> {
        let len = container.len();
        match ix {
            i if i == len => unsafe { Ok(Index::new(ix, container.id())) },
            i if i < len => unsafe {
                Self::vet_inbounds(ix, container)
                    .map(Index::erased)
                    .ok_or(IndexError::Invalid)
            },
            _ => Err(IndexError::OutOfBounds),
        }
    }

    /// Vet an untrusted index for being on item boundaries.
    ///
    /// This assumes a proof that the raw index is inbounds. If you do not
    /// have a proof, use [`vet`][`TrustedItem::vet`] instead, which checks.
    unsafe fn vet_inbounds<'id>(
        ix: u32,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, NonEmpty>>;
}

/// A [`TrustedItem`] where the item is the base unit. Thus, manipulating
/// indices and ranges of the container is as simple as regular arithmetic.
pub unsafe trait TrustedUnit<Array: ?Sized>:
    TrustedItem<Array, Unit = Self> + Sized
where
    Array: TrustedContainer<Item = Self>,
{
}
