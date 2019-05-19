//! Perfect particles are ones that must communicate back to the container for
//! changes. This allows them to index into containers that have items of
//! multiple sizes, such as strings. The container must be inspected to avoid
//! creating an index that points to the middle of an item that takes more
//! than one representational unit, rather than in-between them.
//!
//! In other words, a perfect particle guarantees that it always indexes at an
//! item index, and a simple particle may not do so.

mod index;
mod range;

pub use self::{index::Index, range::Range};
