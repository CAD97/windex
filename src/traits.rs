use {
    crate::{
        proof::{NonEmpty, Unknown},
        Container, Index, IndexError,
    },
    core::{convert::TryFrom, fmt, hash::Hash, ops},
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

/// Types that can back a trusted container: it can have
/// indices and ranges that are trusted to be in bounds.
///
/// It must have a continuously addressable range.
pub unsafe trait TrustedContainer {
    type Item: ?Sized + TrustedItem<Self>;
    type Slice: ?Sized;

    /// The length of the container in base item units.
    fn unit_len(&self) -> usize;

    //    fn begin(&self) -> *const Self::Item;
    //    fn end(&self) -> *const Self::Item;
    //    fn as_slice(&self) -> &Self::Slice;

    unsafe fn get_unchecked(&self, i: usize) -> &Self::Item;
    unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice;
}

/// An item within a trusted container.
pub unsafe trait TrustedItem<Array: TrustedContainer<Item = Self> + ?Sized> {
    type Unit;

    /// Vet an index for being on item boundaries.
    ///
    /// The index for the end of the container should pass.
    /// This function does not imply a nonempty proof.
    fn vet<'id, I: Idx>(
        idx: I,
        container: &Container<'id, Array>,
    ) -> Result<Index<'id, I, Unknown>, IndexError>;

    /// Increment an index to the next item, potentially leaving the container.
    fn after<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, Array>,
    ) -> Index<'id, I, Unknown>;

    /// Advance an index to the next item, if a next item exists.
    fn advance<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, Array>,
    ) -> Option<Index<'id, I, NonEmpty>>;
}

// FUTURE: requires some hairy blanket impls to pass type check
//
// unsafe impl<T: Trustworthy + ?Sized> Trustworthy for &T {
//     type Item = T::Item;
//     type Slice = T::Slice;
//
//     fn begin(&self) -> *const Self::Item {
//         T::begin(self)
//     }
//
//     fn end(&self) -> *const Self::Item {
//         T::end(self)
//     }
//
//     fn as_slice(&self) -> &Self::Slice {
//         T::as_slice(self)
//     }
//
//     unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
//         T::get_unchecked(self, i)
//     }
// }

unsafe impl<T> TrustedContainer for [T] {
    type Item = T;
    type Slice = [T];

    fn unit_len(&self) -> usize {
        self.len()
    }

    unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
        debug_assert!(i < self.len());
        self.get_unchecked(i)
    }

    unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice {
        debug_assert!(r.start <= self.len());
        debug_assert!(r.end <= self.len());
        debug_assert!(r.start <= r.end);
        self.get_unchecked(r)
    }
}

unsafe impl<T> TrustedItem<[T]> for T {
    type Unit = T;

    fn vet<'id, I: Idx>(
        idx: I,
        container: &Container<'id, [T]>,
    ) -> Result<Index<'id, I, Unknown>, IndexError> {
        if idx.as_usize() <= container.unit_len() {
            Ok(unsafe { Index::new(idx) })
        } else {
            Err(IndexError::OutOfBounds)
        }
    }

    fn after<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        _: &Container<'id, [T]>,
    ) -> Index<'id, I, Unknown> {
        unsafe { Index::new(this.untrusted().add(1)) }
    }

    fn advance<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, [T]>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        container.vet(Self::after(this, container).untrusted()).ok()
    }
}

#[cfg(std)]
mod std_impls {
    // Box<impl TrustedContainer>
    // Vec<T>
}
