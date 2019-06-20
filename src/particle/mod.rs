use {
    crate::{proof::*, traits::*, Container},
    core::{convert::TryFrom, ops},
};

pub mod perfect;
pub mod simple;

/// The error returned when failing to construct an arbitrary index.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum IndexError {
    /// The provided raw index was out of bounds of the container.
    OutOfBounds,
    /// The provided raw index was in bounds but not on an item border.
    Invalid,
}

/// A type that can be vetted against a trusted container to create a trusted particle.
pub trait Vettable<'id> {
    type ContainerVetted;
    type RangeVetted;

    fn vet_in_container<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::ContainerVetted, IndexError>
    where
        Array: TrustedContainer;

    fn vet_in_range<P>(
        self,
        range: simple::Range<'id, P>,
    ) -> Option<Self::RangeVetted>;
}

// We impl for the particles' proof parameter separately for type + impl specialization

impl<'id> Vettable<'id> for simple::Index<'id, Unknown> {
    type ContainerVetted = perfect::Index<'id, Unknown>;
    type RangeVetted = simple::Index<'id, NonEmpty>;

    fn vet_in_container<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::ContainerVetted, IndexError>
    where
        Array: TrustedContainer,
    {
        Array::Item::vet(self.untrusted(), container)
    }

    fn vet_in_range<P>(
        self,
        range: simple::Range<'id, P>,
    ) -> Option<Self::RangeVetted> {
        if range.contains(self) {
            Some(unsafe { simple::Index::new(self.untrusted(), self.id()) })
        } else {
            None
        }
    }
}

impl<'id> Vettable<'id> for simple::Index<'id, NonEmpty> {
    type ContainerVetted = perfect::Index<'id, NonEmpty>;
    type RangeVetted = simple::Index<'id, NonEmpty>;

    fn vet_in_container<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::ContainerVetted, IndexError>
    where
        Array: TrustedContainer,
    {
        unsafe { Array::Item::vet_inbounds(self.untrusted(), container).ok_or(IndexError::Invalid) }
    }

    fn vet_in_range<P>(
        self,
        range: simple::Range<'id, P>,
    ) -> Option<Self::RangeVetted> {
        if range.contains(self) {
            Some(self)
        } else {
            None
        }
    }
}

impl<'id> Vettable<'id> for simple::Range<'id, Unknown> {
    type ContainerVetted = perfect::Range<'id, Unknown>;
    type RangeVetted = simple::Range<'id, Unknown>;

    fn vet_in_container<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::ContainerVetted, IndexError>
    where
        Array: TrustedContainer,
    {
        let _end = Vettable::vet_in_container(self.end(), container)?;
        let _start = Vettable::vet_in_container(self.start(), container)?;
        Ok(unsafe { perfect::Range::from(self) })
    }

    fn vet_in_range<P>(
        self,
        range: simple::Range<'id, P>,
    ) -> Option<Self::RangeVetted> {
        if range.contains(self.start()) && self.end() <= range.end() {
            Some(self)
        } else {
            None
        }
    }
}

impl<'id> Vettable<'id> for simple::Range<'id, NonEmpty> {
    type ContainerVetted = perfect::Range<'id, NonEmpty>;
    type RangeVetted = simple::Range<'id, NonEmpty>;

    fn vet_in_container<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::ContainerVetted, IndexError>
    where
        Array: TrustedContainer,
    {
        let _start = Vettable::vet_in_container(self.start(), container)?;
        let _end = Vettable::vet_in_container(self.end(), container)?;
        Ok(unsafe { perfect::Range::from(self) })
    }

    fn vet_in_range<P>(
        self,
        range: simple::Range<'id, P>
    ) -> Option<Self::RangeVetted> {
        if range.contains(self.start()) && self.end() <= range.end() {
            Some(self)
        } else {
            None
        }
    }
}

impl<'id> Vettable<'id> for perfect::Index<'id, Unknown> {
    type ContainerVetted = perfect::Index<'id, NonEmpty>;
    type RangeVetted = simple::Index<'id, NonEmpty>;

    fn vet_in_container<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::ContainerVetted, IndexError>
    where
        Array: TrustedContainer,
    {
        if self < container.end() {
            Ok(unsafe { perfect::Index::new(self.untrusted(), self.id()) })
        } else {
            Err(IndexError::OutOfBounds)
        }
    }

    fn vet_in_range<P>(
        self,
        range: simple::Range<'id, P>
    ) -> Option<Self::RangeVetted> {
        range.vet(self.simple())
    }
}

macro_rules! vettable_int {
    ($($i:tt),* $(,)?) => {$(
        impl<'id> Vettable<'id> for $i {
            type ContainerVetted = perfect::Index<'id, NonEmpty>;
            type RangeVetted = simple::Index<'id, NonEmpty>;

            fn vet_in_container<Array: ?Sized>(
                self,
                container: &Container<'id, Array>,
            ) -> Result<Self::ContainerVetted, IndexError>
            where
                Array: TrustedContainer,
            {
                let ix = u32::try_from(self).map_err(|_| IndexError::OutOfBounds)?;
                if ix < container.len() {
                    unsafe {
                        Array::Item::vet_inbounds(ix, container).ok_or(IndexError::Invalid)
                    }
                } else {
                    Err(IndexError::OutOfBounds)
                }
            }

            fn vet_in_range<P>(
                self,
                range: simple::Range<'id, P>
            ) -> Option<Self::RangeVetted> {
                let ix = u32::try_from(self).ok()?;
                // Safe because we check it immediately
                let index = unsafe { simple::Index::<NonEmpty>::new(ix, range.id()) };
                range.vet(index)
            }
        }

        impl<'id> Vettable<'id> for ops::Range<$i> {
            type ContainerVetted = perfect::Range<'id, Unknown>;
            type RangeVetted = simple::Range<'id, Unknown>;

            fn vet_in_container<Array: ?Sized>(
                self,
                container: &Container<'id, Array>,
            ) -> Result<Self::ContainerVetted, IndexError>
            where
                Array: TrustedContainer,
            {
                let start = u32::try_from(self.start).map_err(|_| IndexError::OutOfBounds)?;
                let end = u32::try_from(self.end).map_err(|_| IndexError::OutOfBounds)?;
                let start = Array::Item::vet(start, container)?;
                let end = Array::Item::vet(end, container)?;
                unsafe {
                    Ok(perfect::Range::new(start.untrusted(), end.untrusted(), container.id()))
                }
            }

            fn vet_in_range<P>(
                self,
                range: simple::Range<'id, P>
            ) -> Option<Self::RangeVetted> {
                let start = u32::try_from(self.start).ok()?;
                let end = u32::try_from(self.end).ok()?;
                // Safe because we check it immediately
                let r = unsafe { simple::Range::<Unknown>::new(start, end, range.id()) };
                range.vet(r)
            }
        }

        impl<'id> Vettable<'id> for ops::RangeTo<$i> {
            type ContainerVetted = perfect::Range<'id, Unknown>;
            type RangeVetted = simple::Range<'id, Unknown>;

            fn vet_in_container<Array: ?Sized>(
                self,
                container: &Container<'id, Array>,
            ) -> Result<Self::ContainerVetted, IndexError>
            where
                Array: TrustedContainer,
            {
                let end = u32::try_from(self.end).map_err(|_| IndexError::OutOfBounds)?;
                let end = Array::Item::vet(end, container)?;
                unsafe {
                    Ok(perfect::Range::new(0, end.untrusted(), container.id()))
                }
            }

            fn vet_in_range<P>(
                self,
                range: simple::Range<'id, P>
            ) -> Option<Self::RangeVetted> {
                let end = u32::try_from(self.end).ok()?;
                // Safe because we check it immediately
                let r = unsafe { simple::Range::<Unknown>::new(range.start().untrusted(), end, range.id()) };
                range.vet(r)
            }
        }

        impl<'id> Vettable<'id> for ops::RangeFrom<$i> {
            type ContainerVetted = perfect::Range<'id, Unknown>;
            type RangeVetted = simple::Range<'id, Unknown>;

            fn vet_in_container<Array: ?Sized>(
                self,
                container: &Container<'id, Array>,
            ) -> Result<Self::ContainerVetted, IndexError>
            where
                Array: TrustedContainer,
            {
                let start = u32::try_from(self.start).map_err(|_| IndexError::OutOfBounds)?;
                let start = Array::Item::vet(start, container)?;
                unsafe {
                    Ok(perfect::Range::new(start.untrusted(), container.len(), container.id()))
                }
            }

            fn vet_in_range<P>(
                self,
                range: simple::Range<'id, P>
            ) -> Option<Self::RangeVetted> {
                let start = u32::try_from(self.start).ok()?;
                // Safe because we check it immediately
                let r = unsafe { simple::Range::<Unknown>::new(start, range.end().untrusted(), range.id()) };
                range.vet(r)
            }
        }
    )*};
}

vettable_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
