pub mod error;
pub mod trace;
pub mod inflector;

// Convenience re-exports so downstream crates depend only on doido-core.
pub use ::anyhow;
pub use ::thiserror;
pub use ::async_trait::async_trait;
pub use ::serde;
pub use ::tracing;

pub use error::{Result, AnyhowContext};
pub use inflector::{Inflector, Inflections, init_inflections};
