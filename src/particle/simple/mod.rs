//! Simple particles are ones that can be manipulated without communicating
//! with the container; the item is the same as the representational unit,
//! so getting the next item is as simple as a `+1`. Creating and manipulating
//! a simple particle from a perfect one is always safe; however, using one
//! requires either the item to be a trusted unit, or upgrading the particle
//! against the container.
//!
//! In other words, a simple particle is only guaranteed to be inbounds of the
//! backing container. A perfect particle is guaranteed on item boundaries.

mod index;
mod range;

pub use self::{index::Index, range::Range};
