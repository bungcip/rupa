//! Rupa compiler crate: name resolution, semantic analysis, and lowering to IR.

#![warn(missing_docs)]

pub mod compile;
pub mod document;
pub mod error;
pub mod group;
pub mod layer;
pub mod lower;
pub mod metadata;
pub mod paint;
pub mod prng;
pub mod random;
pub mod rng;
pub mod shapes;
pub mod validation;

#[cfg(test)]
mod test_support;

/// Re-export commonly used types
pub use compile::*;
pub use document::*;
pub use error::*;
pub use group::*;
pub use layer::*;
pub use lower::*;
pub use metadata::metadata_json;
pub use paint::*;
pub use prng::Prng;
pub use shapes::*;
pub use validation::*;
