# doido-core — Spec

Rails analogue: **Active Support**

## Decisions (resolved in interview)

- **Error strategy: `thiserror` per crate, `anyhow` at app level**
  - Each crate owns a typed error enum via `thiserror`
  - All implement `std::error::Error`, so `?` propagates into `anyhow::Error` in app code
  - `doido-core` defines no umbrella error — only a `Result<T>` alias using `anyhow`
- **All string inflection transformations ship** — plus a custom inflection rules file

## Error Convention Per Crate

Each crate defines its own error type:

```rust
// Pattern every crate follows
#[derive(thiserror::Error, Debug)]
pub enum RouterError { ... }

#[derive(thiserror::Error, Debug)]
pub enum ControllerError { ... }

#[derive(thiserror::Error, Debug)]
pub enum ModelError { ... }

#[derive(thiserror::Error, Debug)]
pub enum ViewError { ... }

// etc.
```

`doido-core` re-exports `anyhow` and `thiserror` for convenience so crates only depend on `doido-core`:

```rust
pub use anyhow::{self, anyhow, bail, Context as AnyhowContext};
pub use thiserror;

/// App-level result type — used in controllers, jobs, and application code
pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;
```

## Inflector — All Transformations

```rust
use doido_core::inflector::Inflector;

// Standard transformations
Inflector::pluralize("post")           // → "posts"
Inflector::pluralize("person")         // → "people"
Inflector::singularize("posts")        // → "post"
Inflector::camelize("post_comment")    // → "PostComment"
Inflector::camelize_lower("post_comment") // → "postComment"
Inflector::underscore("PostComment")   // → "post_comment"
Inflector::dasherize("post_comment")   // → "post-comment"
Inflector::humanize("post_comment")    // → "Post comment"
Inflector::tableize("PostComment")     // → "post_comments"
Inflector::classify("post_comments")   // → "PostComment"
Inflector::foreign_key("PostComment")  // → "post_comment_id"
Inflector::constantize("post_comment") // → "POST_COMMENT"
```

Used by:
- `doido-generators` — derives file names, table names, module names from user input
- `doido-router` — derives route names from controller names
- `doido-cli` — `doido routes` table formatting

## Custom Inflection Rules — `config/inflections.rs`

Users override or extend the default rules in `config/inflections.rs`:

```rust
// config/inflections.rs
use doido_core::inflector::Inflections;

pub fn configure(inflections: &mut Inflections) {
    // Override irregular singular/plural
    inflections.irregular("person", "people");
    inflections.irregular("goose", "geese");

    // Uncountable words (same singular and plural)
    inflections.uncountable("sheep");
    inflections.uncountable("fish");
    inflections.uncountable("money");

    // Custom plural rules (regex pattern, replacement)
    inflections.plural(r"(quiz)$", "${1}zes");

    // Custom singular rules
    inflections.singular(r"(quiz)zes$", "${1}");

    // Acronyms (preserved in camelize/underscore)
    inflections.acronym("API");
    inflections.acronym("HTML");
    inflections.acronym("HTTP");
}
```

`config/inflections.rs` is loaded at boot by the framework. `doido-generators` generates this file as part of `doido new <app>`.

## Module Structure

```
doido-core/
  src/
    lib.rs
    error.rs          ← Result<T> alias, re-exports thiserror + anyhow
    inflector/
      mod.rs           ← Inflector struct + all transformation methods
      rules.rs         ← default English inflection rules
      inflections.rs   ← Inflections config struct (user-facing)
    async_trait.rs    ← re-export async_trait for convenience
    tracing.rs        ← tracing span helpers used across crates
```

## Tracing Helpers

Thin wrappers so crates emit consistent structured events without duplicating setup:

```rust
doido_core::trace::request(method, path, status, latency_ms);
doido_core::trace::job(job_name, queue, attempt, result);
doido_core::trace::query(sql, duration_ms);
doido_core::trace::mail(to, subject, deliverer);
```

## Known Requirements

- `doido-core` is a **leaf dependency** — depends on nothing else in the workspace
- Re-exports: `anyhow`, `thiserror`, `async_trait`, `serde`, `tracing`
- `Result<T>` alias is `anyhow::Result<T>` — used in app-level code
- All inflection transformations implemented and tested for English by default
- Custom inflection file loaded at app boot; overrides default rules
- Acronym support in `camelize` / `underscore` (e.g. `APIClient` ↔ `api_client`)

## TDD Surface

- Test each inflection transformation with standard English cases
- Test irregular overrides (`person` → `people`)
- Test uncountable words return same form for singular and plural
- Test custom plural/singular regex rules apply correctly
- Test acronym preservation in `camelize` and `underscore`
- Test `config/inflections.rs` overrides take precedence over defaults
- Test `tableize` and `classify` are inverses of each other
- Test `foreign_key` output matches expected convention
- Test `Result<T>` propagates `?` from a `thiserror` crate error into `anyhow::Error`
- Test tracing helpers emit events with correct structured fields
