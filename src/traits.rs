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

unsafe impl<T: TrustedContainer + ?Sized> TrustedContainer for &T {
    type Item = T::Item;
    type Slice = T::Slice;

    fn unit_len(&self) -> usize {
        T::unit_len(self)
    }

    unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
        T::get_unchecked(self, i)
    }

    unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice {
        T::slice_unchecked(self, r)
    }
}

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

unsafe impl<T: TrustedItem<Array> + ?Sized, Array: TrustedContainer<Item = T> + ?Sized>
    TrustedItem<&Array> for T
{
    type Unit = T::Unit;

    fn vet<'id, I: Idx>(
        idx: I,
        container: &Container<'id, &Array>,
    ) -> Result<Index<'id, I, Unknown>, IndexError> {
        T::vet(idx, container.project())
    }

    fn after<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, &Array>,
    ) -> Index<'id, I, Unknown> {
        T::after(this, container.project())
    }

    fn advance<'id, I: Idx>(
        this: Index<'id, I, NonEmpty>,
        container: &Container<'id, &Array>,
    ) -> Option<Index<'id, I, NonEmpty>> {
        T::advance(this, container.project())
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

#[cfg(feature = "std")]
mod std_impls {
    use super::*;
    use std::{boxed::Box, vec::Vec};

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    unsafe impl<T: TrustedContainer + ?Sized> TrustedContainer for Box<T> {
        type Item = T::Item;
        type Slice = T::Slice;

        fn unit_len(&self) -> usize {
            T::unit_len(&self)
        }

        unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
            T::get_unchecked(self, i)
        }

        unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice {
            T::slice_unchecked(self, r)
        }
    }

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    unsafe impl<T: TrustedItem<Array> + ?Sized, Array: TrustedContainer<Item = T> + ?Sized>
        TrustedItem<Box<Array>> for T
    {
        type Unit = T::Unit;

        fn vet<'id, I: Idx>(
            idx: I,
            container: &Container<'id, Box<Array>>,
        ) -> Result<Index<'id, I, Unknown>, IndexError> {
            T::vet(idx, container.project())
        }

        fn after<'id, I: Idx>(
            this: Index<'id, I, NonEmpty>,
            container: &Container<'id, Box<Array>>,
        ) -> Index<'id, I, Unknown> {
            T::after(this, container.project())
        }

        fn advance<'id, I: Idx>(
            this: Index<'id, I, NonEmpty>,
            container: &Container<'id, Box<Array>>,
        ) -> Option<Index<'id, I, NonEmpty>> {
            T::advance(this, container.project())
        }
    }

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    impl<'id, Array: TrustedContainer + ?Sized> Container<'id, Box<Array>> {
        pub fn project(&self) -> &Container<'id, Array> {
            unsafe { &*(&**self.untrusted() as *const Array as *const Container<'id, Array>) }
        }
    }

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    unsafe impl<T> TrustedContainer for Vec<T> {
        type Item = T;
        type Slice = [T];

        fn unit_len(&self) -> usize {
            self.len()
        }

        unsafe fn get_unchecked(&self, i: usize) -> &Self::Item {
            <[T]>::get_unchecked(self, i)
        }

        unsafe fn slice_unchecked(&self, r: ops::Range<usize>) -> &Self::Slice {
            <[T]>::slice_unchecked(self, r)
        }
    }

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    unsafe impl<T: TrustedItem<[T]>> TrustedItem<Vec<T>> for T {
        type Unit = T::Unit;

        fn vet<'id, I: Idx>(
            idx: I,
            container: &Container<'id, Vec<T>>,
        ) -> Result<Index<'id, I, Unknown>, IndexError> {
            T::vet(idx, container.project())
        }

        fn after<'id, I: Idx>(
            this: Index<'id, I, NonEmpty>,
            container: &Container<'id, Vec<T>>,
        ) -> Index<'id, I, Unknown> {
            T::after(this, container.project())
        }

        fn advance<'id, I: Idx>(
            this: Index<'id, I, NonEmpty>,
            container: &Container<'id, Vec<T>>,
        ) -> Option<Index<'id, I, NonEmpty>> {
            T::advance(this, container.project())
        }
    }

    #[cfg_attr(feature = "doc", doc(cfg(feature = "std")))]
    impl<'id, T> Container<'id, Vec<T>> {
        pub fn project(&self) -> &Container<'id, [T]> {
            unsafe { &*(&**self.untrusted() as *const [T] as *const Container<'id, [T]>) }
        }
    }
}
