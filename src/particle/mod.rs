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
    type Vetted;

    fn vet<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::Vetted, IndexError>
    where
        Array: TrustedContainer;
}

// We impl for the particles' proof parameter separately for type + impl specialization

impl<'id> Vettable<'id> for simple::Index<'id, Unknown> {
    type Vetted = perfect::Index<'id, Unknown>;

    fn vet<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::Vetted, IndexError>
    where
        Array: TrustedContainer,
    {
        Array::Item::vet(self.untrusted(), container)
    }
}

impl<'id> Vettable<'id> for simple::Index<'id, NonEmpty> {
    type Vetted = perfect::Index<'id, NonEmpty>;

    fn vet<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::Vetted, IndexError>
    where
        Array: TrustedContainer,
    {
        unsafe { Array::Item::vet_inbounds(self.untrusted(), container).ok_or(IndexError::Invalid) }
    }
}

impl<'id> Vettable<'id> for simple::Range<'id, Unknown> {
    type Vetted = perfect::Range<'id, Unknown>;

    fn vet<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::Vetted, IndexError>
    where
        Array: TrustedContainer,
    {
        let _end = Vettable::vet(self.end(), container)?;
        let _start = Vettable::vet(self.start(), container)?;
        Ok(unsafe { perfect::Range::from(self) })
    }
}

impl<'id> Vettable<'id> for simple::Range<'id, NonEmpty> {
    type Vetted = perfect::Range<'id, NonEmpty>;

    fn vet<Array: ?Sized>(
        self,
        container: &Container<'id, Array>,
    ) -> Result<Self::Vetted, IndexError>
    where
        Array: TrustedContainer,
    {
        let _start = Vettable::vet(self.start(), container)?;
        let _end = Vettable::vet(self.end(), container)?;
        Ok(unsafe { perfect::Range::from(self) })
    }
}

macro_rules! vettable_int {
    ($($i:tt),* $(,)?) => {$(
        impl<'id> Vettable<'id> for $i {
            type Vetted = perfect::Index<'id, NonEmpty>;

            fn vet<Array: ?Sized>(
                self,
                container: &Container<'id, Array>,
            ) -> Result<Self::Vetted, IndexError>
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
        }

        impl<'id> Vettable<'id> for ops::Range<$i> {
            type Vetted = perfect::Range<'id, Unknown>;

            fn vet<Array: ?Sized>(
                self,
                container: &Container<'id, Array>,
            ) -> Result<Self::Vetted, IndexError>
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
        }

        impl<'id> Vettable<'id> for ops::RangeTo<$i> {
            type Vetted = perfect::Range<'id, Unknown>;

            fn vet<Array: ?Sized>(
                self,
                container: &Container<'id, Array>,
            ) -> Result<Self::Vetted, IndexError>
            where
                Array: TrustedContainer,
            {
                let end = u32::try_from(self.end).map_err(|_| IndexError::OutOfBounds)?;
                let end = Array::Item::vet(end, container)?;
                unsafe {
                    Ok(perfect::Range::new(0, end.untrusted(), container.id()))
                }
            }
        }

        impl<'id> Vettable<'id> for ops::RangeFrom<$i> {
            type Vetted = perfect::Range<'id, Unknown>;

            fn vet<Array: ?Sized>(
                self,
                container: &Container<'id, Array>,
            ) -> Result<Self::Vetted, IndexError>
            where
                Array: TrustedContainer,
            {
                let start = u32::try_from(self.start).map_err(|_| IndexError::OutOfBounds)?;
                let start = Array::Item::vet(start, container)?;
                unsafe {
                    Ok(perfect::Range::new(start.untrusted(), container.len(), container.id()))
                }
            }
        }
    )*};
}

vettable_int!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
