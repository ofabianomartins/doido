pub use ::anyhow::{self, anyhow, bail, Context as AnyhowContext};
pub use ::thiserror;

/// App-level result type.
/// Use in controllers, jobs, and application code.
/// Crate-level errors use their own typed enums via `thiserror`.
pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;
