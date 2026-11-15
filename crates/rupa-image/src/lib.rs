//! Rupa image crate: image resources and pixel buffers.
//!
//! Images are decoded to canonical straight-alpha RGBA8 buffers. PNG is the
//! supported input format (specification §10). [`ResourceStore`] resolves
//! resource references against a project directory and caches decoded images.

#![warn(missing_docs)]

mod cache;

pub mod decode;
pub mod encode;
pub mod font;
pub mod resource;

/// Re-export commonly used types
pub use decode::*;
pub use encode::*;
pub use font::*;
pub use resource::*;
