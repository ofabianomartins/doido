# doido-core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement `doido-core` — the workspace's leaf dependency providing a `Result<T>` alias, shared re-exports (`anyhow`, `thiserror`, `async_trait`, `serde`, `tracing`), structured trace helpers, and a fully tested English inflection engine with a user-configurable rules API.

**Architecture:** Single crate with zero workspace dependencies. Three subsystems: (1) `error` module — `Result<T>` alias + `anyhow`/`thiserror` re-exports so downstream crates have one dependency; (2) `trace` module — thin wrappers around `tracing::info!` emitting consistent structured events; (3) `inflector` subsystem — an `Inflections` struct that holds regex rules + a static `Inflector` facade backed by a `OnceLock<Inflections>` global, initialised once at app boot via `init_inflections`.

**Tech Stack:** Rust, `anyhow 1`, `thiserror 1`, `async-trait 0.1`, `tracing 0.1`, `serde 1`, `regex 1`; dev: `tracing-test 0.2`

---

## File Structure

| File | Purpose |
|------|---------|
| `doido-core/Cargo.toml` | Crate manifest with all dependencies |
| `doido-core/src/lib.rs` | Module declarations + top-level re-exports |
| `doido-core/src/error.rs` | `Result<T>` alias; re-export `anyhow`, `thiserror` |
| `doido-core/src/trace.rs` | `request`, `job`, `query`, `mail` structured event helpers |
| `doido-core/src/inflector/mod.rs` | `Inflector` static facade; `INFLECTIONS: OnceLock`; `init_inflections` |
| `doido-core/src/inflector/inflections.rs` | `Inflections` struct + all transformation methods |
| `doido-core/src/inflector/rules.rs` | `defaults() -> Inflections` — English plural/singular/irregular rules |
| `doido-core/tests/error_test.rs` | `Result<T>` + `?` propagation tests |
| `doido-core/tests/trace_test.rs` | Structured event emission tests |
| `doido-core/tests/inflector_test.rs` | All inflection tests (transformations, rules, overrides) |

---

### Task 1: Crate Scaffold

**Files:**
- Create: `doido-core/Cargo.toml`
- Create: `doido-core/src/lib.rs`

- [ ] **Step 1: Create `doido-core/Cargo.toml`**

```toml
[package]
name = "doido-core"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1"
thiserror = "1"
async-trait = "0.1"
tracing = "0.1"
serde = { version = "1", features = ["derive"] }
regex = "1"

[dev-dependencies]
tracing-test = "0.2"
```

- [ ] **Step 2: Create `doido-core/src/lib.rs`**

```rust
pub mod error;
pub mod trace;
pub mod inflector;

pub use error::{Result, AnyhowContext};
pub use ::async_trait::async_trait;
pub use ::serde;
pub use ::tracing;
pub use ::anyhow;
pub use ::thiserror;
```

- [ ] **Step 3: Create stub files so the crate compiles**

Create `doido-core/src/error.rs`:
```rust
// filled in Task 2
```

Create `doido-core/src/trace.rs`:
```rust
// filled in Task 3
```

Create `doido-core/src/inflector/mod.rs`:
```rust
// filled in Task 9
```

Create `doido-core/src/inflector/inflections.rs`:
```rust
// filled in Task 4
```

Create `doido-core/src/inflector/rules.rs`:
```rust
// filled in Task 5
```

- [ ] **Step 4: Verify the crate is visible to the workspace**

Run: `cargo check -p doido-core`
Expected: errors about empty modules — that is fine; it confirms the crate is found.

- [ ] **Step 5: Commit**

```bash
git add doido-core/
git commit -m "feat(core): add doido-core crate scaffold"
```

---

### Task 2: Error Module

**Files:**
- Create: `doido-core/src/error.rs`
- Create: `doido-core/tests/error_test.rs`

- [ ] **Step 1: Write the failing test**

Create `doido-core/tests/error_test.rs`:

```rust
use doido_core::error::{Result, AnyhowContext};

// Define a typed error as any downstream crate would
#[derive(doido_core::thiserror::Error, Debug)]
enum FakeError {
    #[error("something went wrong: {0}")]
    Oops(String),
}

fn might_fail() -> std::result::Result<(), FakeError> {
    Err(FakeError::Oops("bad".into()))
}

fn propagate_via_question_mark() -> Result<()> {
    might_fail()?; // ? converts FakeError into anyhow::Error
    Ok(())
}

#[test]
fn test_thiserror_propagates_into_anyhow_result() {
    let result = propagate_via_question_mark();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("something went wrong: bad"));
}

#[test]
fn test_anyhow_context_adds_message() {
    let result: Result<()> = might_fail()
        .map_err(|e| doido_core::anyhow::anyhow!(e))
        .with_context(|| "extra context");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("extra context"), "got: {msg}");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p doido-core --test error_test`
Expected: FAIL — `error` module is empty.

- [ ] **Step 3: Implement `doido-core/src/error.rs`**

```rust
pub use anyhow::{self, anyhow, bail, Context as AnyhowContext};
pub use thiserror;

/// App-level result type.
/// Use in controllers, jobs, and application code.
/// Crate-level errors use their own typed enums via `thiserror`.
pub type Result<T, E = anyhow::Error> = std::result::Result<T, E>;
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p doido-core --test error_test`
Expected: PASS — 2 tests.

- [ ] **Step 5: Commit**

```bash
git add doido-core/src/error.rs doido-core/tests/error_test.rs
git commit -m "feat(core): add Result<T> alias and anyhow/thiserror re-exports"
```

---

### Task 3: Trace Helpers

**Files:**
- Create: `doido-core/src/trace.rs`
- Create: `doido-core/tests/trace_test.rs`

- [ ] **Step 1: Write the failing tests**

Create `doido-core/tests/trace_test.rs`:

```rust
use tracing_test::traced_test;

#[test]
#[traced_test]
fn test_request_emits_structured_event() {
    doido_core::trace::request("GET", "/posts", 200, 42);
    assert!(logs_contain("request"));
    assert!(logs_contain("GET"));
    assert!(logs_contain("200"));
    assert!(logs_contain("42"));
}

#[test]
#[traced_test]
fn test_job_emits_structured_event() {
    doido_core::trace::job("ProcessPayment", "default", 1, "ok");
    assert!(logs_contain("job"));
    assert!(logs_contain("ProcessPayment"));
    assert!(logs_contain("default"));
}

#[test]
#[traced_test]
fn test_query_emits_structured_event() {
    doido_core::trace::query("SELECT * FROM posts", 5);
    assert!(logs_contain("query"));
    assert!(logs_contain("SELECT * FROM posts"));
    assert!(logs_contain("5"));
}

#[test]
#[traced_test]
fn test_mail_emits_structured_event() {
    doido_core::trace::mail("user@example.com", "Welcome!", "smtp");
    assert!(logs_contain("mail"));
    assert!(logs_contain("user@example.com"));
    assert!(logs_contain("Welcome!"));
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p doido-core --test trace_test`
Expected: FAIL — `trace` module is empty.

- [ ] **Step 3: Implement `doido-core/src/trace.rs`**

```rust
pub fn request(method: &str, path: &str, status: u16, latency_ms: u64) {
    tracing::info!(method, path, status, latency_ms, "request");
}

pub fn job(job_name: &str, queue: &str, attempt: u32, result: &str) {
    tracing::info!(job_name, queue, attempt, result, "job");
}

pub fn query(sql: &str, duration_ms: u64) {
    tracing::info!(sql, duration_ms, "query");
}

pub fn mail(to: &str, subject: &str, deliverer: &str) {
    tracing::info!(to, subject, deliverer, "mail");
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p doido-core --test trace_test`
Expected: PASS — 4 tests.

- [ ] **Step 5: Commit**

```bash
git add doido-core/src/trace.rs doido-core/tests/trace_test.rs
git commit -m "feat(core): add structured trace helpers for request/job/query/mail"
```

---

### Task 4: Inflections Config Struct

**Files:**
- Create: `doido-core/src/inflector/inflections.rs`

This task builds the `Inflections` data structure. Transformation methods come later.

- [ ] **Step 1: Write the failing test (inline, inside inflections.rs)**

Replace `doido-core/src/inflector/inflections.rs` with:

```rust
use regex::Regex;
use std::collections::HashSet;

pub struct Inflections {
    /// Plural rules — (pattern, replacement). Tried in reverse-add order (last added = highest priority).
    pub(crate) plurals: Vec<(Regex, String)>,
    /// Singular rules — same ordering convention as plurals.
    pub(crate) singulars: Vec<(Regex, String)>,
    /// Irregular pairs — (singular, plural), stored lowercase.
    pub(crate) irregulars: Vec<(String, String)>,
    pub(crate) uncountables: HashSet<String>,
    /// Acronyms preserved verbatim in camelize (e.g. "API", "HTML").
    pub(crate) acronyms: HashSet<String>,
}

impl Inflections {
    pub fn new() -> Self {
        Self {
            plurals: Vec::new(),
            singulars: Vec::new(),
            irregulars: Vec::new(),
            uncountables: HashSet::new(),
            acronyms: HashSet::new(),
        }
    }

    /// Add a plural regex rule. `pattern` is a Rust regex; `replacement` uses `$1`/`${1}` syntax.
    pub fn plural(&mut self, pattern: &str, replacement: &str) {
        self.plurals.push((
            Regex::new(pattern).unwrap_or_else(|e| panic!("invalid plural pattern `{pattern}`: {e}")),
            replacement.to_string(),
        ));
    }

    /// Add a singular regex rule.
    pub fn singular(&mut self, pattern: &str, replacement: &str) {
        self.singulars.push((
            Regex::new(pattern).unwrap_or_else(|e| panic!("invalid singular pattern `{pattern}`: {e}")),
            replacement.to_string(),
        ));
    }

    /// Register an irregular singular/plural pair (case-insensitive matching).
    pub fn irregular(&mut self, singular: &str, plural: &str) {
        self.irregulars.push((singular.to_lowercase(), plural.to_lowercase()));
    }

    /// Mark a word as uncountable (same form for singular and plural).
    pub fn uncountable(&mut self, word: &str) {
        self.uncountables.insert(word.to_lowercase());
    }

    /// Register an acronym that is preserved verbatim in `camelize` and handled in `underscore`.
    pub fn acronym(&mut self, word: &str) {
        self.acronyms.insert(word.to_uppercase());
    }
}

impl Default for Inflections {
    fn default() -> Self {
        crate::inflector::rules::defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plural_adds_rule() {
        let mut i = Inflections::new();
        i.plural(r"s$", "ses");
        assert_eq!(i.plurals.len(), 1);
    }

    #[test]
    fn test_singular_adds_rule() {
        let mut i = Inflections::new();
        i.singular(r"ses$", "s");
        assert_eq!(i.singulars.len(), 1);
    }

    #[test]
    fn test_irregular_stores_lowercase() {
        let mut i = Inflections::new();
        i.irregular("Person", "People");
        assert_eq!(i.irregulars[0], ("person".to_string(), "people".to_string()));
    }

    #[test]
    fn test_uncountable_stores_lowercase() {
        let mut i = Inflections::new();
        i.uncountable("Sheep");
        assert!(i.uncountables.contains("sheep"));
    }

    #[test]
    fn test_acronym_stores_uppercase() {
        let mut i = Inflections::new();
        i.acronym("api");
        assert!(i.acronyms.contains("API"));
    }
}
```

- [ ] **Step 2: Update `doido-core/src/inflector/mod.rs` to declare the submodules**

```rust
pub mod inflections;
pub(crate) mod rules;

pub use inflections::Inflections;
```

- [ ] **Step 3: Add a stub `rules.rs` that returns an empty `Inflections`**

```rust
use super::inflections::Inflections;

pub fn defaults() -> Inflections {
    Inflections::new()
}
```

- [ ] **Step 4: Run the inline tests to verify they pass**

Run: `cargo test -p doido-core inflections`
Expected: PASS — 5 tests.

- [ ] **Step 5: Commit**

```bash
git add doido-core/src/inflector/
git commit -m "feat(core): add Inflections config struct with plural/singular/irregular/uncountable/acronym methods"
```

---

### Task 5: Default English Rules

**Files:**
- Modify: `doido-core/src/inflector/rules.rs`

Rules are added lowest-priority first. When applied, the `Vec` is iterated in reverse (last added = highest priority = tried first).

- [ ] **Step 1: No test to write** — rules are exercised via pluralize/singularize tests in Task 6.

- [ ] **Step 2: Replace `doido-core/src/inflector/rules.rs` with the full default ruleset**

```rust
use super::inflections::Inflections;

pub fn defaults() -> Inflections {
    let mut i = Inflections::new();

    // ── Plural rules ──────────────────────────────────────────────────────────
    // Added lowest-priority first; last-added rule is tried first.

    i.plural(r"$", "s");                               // catch-all: word → words
    i.plural(r"(s|x|z|ch|sh)$", "${1}es");            // box→boxes, watch→watches
    i.plural(r"([^aeiouy])y$", "${1}ies");             // city→cities  (vowel+y stays: day→days via catch-all)
    i.plural(r"(tomat|potat)o$", "${1}oes");           // tomato→tomatoes
    i.plural(r"sis$", "ses");                          // analysis→analyses
    i.plural(r"([ti])um$", "${1}a");                   // datum→data, medium→media
    i.plural(r"(quiz)$", "${1}zes");                   // quiz→quizzes

    // ── Singular rules ────────────────────────────────────────────────────────
    // Added lowest-priority first.

    i.singular(r"s$", "");                             // catch-all: dogs→dog
    i.singular(r"(ss|us|is)$", "${1}");                // class→class, radius→radius, analysis→analysis
    i.singular(r"(x|ch|ss|sh)es$", "${1}");            // boxes→box, watches→watch
    i.singular(r"([^aeiouy])ies$", "${1}y");           // cities→city
    i.singular(r"(tomat|potat)oes$", "${1}o");         // potatoes→potato
    i.singular(r"ses$", "sis");                        // analyses→analysis
    i.singular(r"([ti])a$", "${1}um");                 // data→datum

    // ── Default irregulars ────────────────────────────────────────────────────

    i.irregular("person", "people");
    i.irregular("man", "men");
    i.irregular("child", "children");
    i.irregular("move", "moves");
    i.irregular("zombie", "zombies");

    // ── Default uncountables ──────────────────────────────────────────────────

    for word in &[
        "equipment", "information", "rice", "money", "species",
        "series", "fish", "sheep", "jeans", "police",
    ] {
        i.uncountable(word);
    }

    i
}
```

- [ ] **Step 3: Verify the crate still compiles**

Run: `cargo check -p doido-core`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add doido-core/src/inflector/rules.rs
git commit -m "feat(core): add default English inflection rules"
```

---

### Task 6: Pluralize and Singularize

**Files:**
- Modify: `doido-core/src/inflector/inflections.rs`
- Create: `doido-core/tests/inflector_test.rs` (first batch)

- [ ] **Step 1: Write the failing tests**

Create `doido-core/tests/inflector_test.rs`:

```rust
use doido_core::inflector::Inflections;

// ── pluralize ──────────────────────────────────────────────────────────────

#[test]
fn test_pluralize_regular_word() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("post"), "posts");
}

#[test]
fn test_pluralize_sibilant_ending() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("box"), "boxes");
    assert_eq!(i.pluralize("watch"), "watches");
    assert_eq!(i.pluralize("dish"), "dishes");
}

#[test]
fn test_pluralize_y_ending() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("city"), "cities");
    assert_eq!(i.pluralize("day"), "days"); // vowel+y: catch-all applies
}

#[test]
fn test_pluralize_irregular() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("person"), "people");
    assert_eq!(i.pluralize("man"), "men");
    assert_eq!(i.pluralize("child"), "children");
}

#[test]
fn test_pluralize_uncountable() {
    let i = Inflections::default();
    assert_eq!(i.pluralize("sheep"), "sheep");
    assert_eq!(i.pluralize("money"), "money");
    assert_eq!(i.pluralize("fish"), "fish");
}

// ── singularize ────────────────────────────────────────────────────────────

#[test]
fn test_singularize_regular_word() {
    let i = Inflections::default();
    assert_eq!(i.singularize("posts"), "post");
}

#[test]
fn test_singularize_es_ending() {
    let i = Inflections::default();
    assert_eq!(i.singularize("boxes"), "box");
    assert_eq!(i.singularize("watches"), "watch");
    assert_eq!(i.singularize("dishes"), "dish");
}

#[test]
fn test_singularize_ies_ending() {
    let i = Inflections::default();
    assert_eq!(i.singularize("cities"), "city");
}

#[test]
fn test_singularize_irregular() {
    let i = Inflections::default();
    assert_eq!(i.singularize("people"), "person");
    assert_eq!(i.singularize("men"), "man");
    assert_eq!(i.singularize("children"), "child");
}

#[test]
fn test_singularize_uncountable() {
    let i = Inflections::default();
    assert_eq!(i.singularize("sheep"), "sheep");
    assert_eq!(i.singularize("money"), "money");
}

// ── custom rules override defaults ────────────────────────────────────────

#[test]
fn test_custom_irregular_overrides_default() {
    let mut i = Inflections::default();
    i.irregular("goose", "geese");
    assert_eq!(i.pluralize("goose"), "geese");
    assert_eq!(i.singularize("geese"), "goose");
}

#[test]
fn test_custom_uncountable() {
    let mut i = Inflections::default();
    i.uncountable("news");
    assert_eq!(i.pluralize("news"), "news");
    assert_eq!(i.singularize("news"), "news");
}

#[test]
fn test_custom_plural_regex_rule() {
    let mut i = Inflections::default();
    i.plural(r"(quiz)$", "${1}zes");
    assert_eq!(i.pluralize("quiz"), "quizzes");
}

#[test]
fn test_custom_singular_regex_rule() {
    let mut i = Inflections::default();
    i.singular(r"(quiz)zes$", "${1}");
    assert_eq!(i.singularize("quizzes"), "quiz");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p doido-core --test inflector_test`
Expected: FAIL — methods don't exist yet.

- [ ] **Step 3: Add `pluralize` and `singularize` to `Inflections`**

Add these methods inside the `impl Inflections` block in `inflections.rs`:

```rust
/// Returns the plural form of `word` according to the configured rules.
pub fn pluralize(&self, word: &str) -> String {
    let lower = word.to_lowercase();

    // 1. Uncountables are returned unchanged.
    if self.uncountables.contains(&lower) {
        return word.to_string();
    }

    // 2. Irregulars take priority over regex rules.
    for (singular, plural) in &self.irregulars {
        if singular == &lower {
            return plural.clone();
        }
        if plural == &lower {
            // Already plural — return as-is.
            return plural.clone();
        }
    }

    // 3. Regex rules — tried in reverse add-order (last added = highest priority).
    for (pattern, replacement) in self.plurals.iter().rev() {
        if pattern.is_match(&lower) {
            return pattern.replace(&lower, replacement.as_str()).to_string();
        }
    }

    word.to_string()
}

/// Returns the singular form of `word` according to the configured rules.
pub fn singularize(&self, word: &str) -> String {
    let lower = word.to_lowercase();

    // 1. Uncountables.
    if self.uncountables.contains(&lower) {
        return word.to_string();
    }

    // 2. Irregulars — lookup by plural form.
    for (singular, plural) in &self.irregulars {
        if plural == &lower {
            return singular.clone();
        }
        if singular == &lower {
            // Already singular — return as-is.
            return singular.clone();
        }
    }

    // 3. Regex rules.
    for (pattern, replacement) in self.singulars.iter().rev() {
        if pattern.is_match(&lower) {
            return pattern.replace(&lower, replacement.as_str()).to_string();
        }
    }

    word.to_string()
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p doido-core --test inflector_test`
Expected: PASS — 14 tests.

- [ ] **Step 5: Commit**

```bash
git add doido-core/src/inflector/inflections.rs doido-core/tests/inflector_test.rs
git commit -m "feat(core): implement pluralize/singularize with regex rules, irregulars, and uncountables"
```

---

### Task 7: Basic String Transformations

**Files:**
- Modify: `doido-core/src/inflector/inflections.rs`
- Modify: `doido-core/tests/inflector_test.rs`

- [ ] **Step 1: Append these tests to `doido-core/tests/inflector_test.rs`**

```rust
// ── camelize ──────────────────────────────────────────────────────────────

#[test]
fn test_camelize_snake_case() {
    let i = Inflections::default();
    assert_eq!(i.camelize("post_comment"), "PostComment");
    assert_eq!(i.camelize("active_record"), "ActiveRecord");
}

#[test]
fn test_camelize_already_camel() {
    let i = Inflections::default();
    assert_eq!(i.camelize("PostComment"), "PostComment");
}

#[test]
fn test_camelize_lower() {
    let i = Inflections::default();
    assert_eq!(i.camelize_lower("post_comment"), "postComment");
    assert_eq!(i.camelize_lower("active_record"), "activeRecord");
}

// ── underscore ────────────────────────────────────────────────────────────

#[test]
fn test_underscore_camel_case() {
    let i = Inflections::default();
    assert_eq!(i.underscore("PostComment"), "post_comment");
    assert_eq!(i.underscore("ActiveRecord"), "active_record");
}

#[test]
fn test_underscore_already_snake() {
    let i = Inflections::default();
    assert_eq!(i.underscore("post_comment"), "post_comment");
}

// ── dasherize ─────────────────────────────────────────────────────────────

#[test]
fn test_dasherize() {
    let i = Inflections::default();
    assert_eq!(i.dasherize("post_comment"), "post-comment");
    assert_eq!(i.dasherize("active_record"), "active-record");
}

// ── humanize ─────────────────────────────────────────────────────────────

#[test]
fn test_humanize_snake_case() {
    let i = Inflections::default();
    assert_eq!(i.humanize("post_comment"), "Post comment");
}

#[test]
fn test_humanize_strips_id_suffix() {
    let i = Inflections::default();
    assert_eq!(i.humanize("author_id"), "Author");
}

// ── constantize ───────────────────────────────────────────────────────────

#[test]
fn test_constantize() {
    let i = Inflections::default();
    assert_eq!(i.constantize("post_comment"), "POST_COMMENT");
    assert_eq!(i.constantize("active-record"), "ACTIVE_RECORD");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p doido-core --test inflector_test`
Expected: FAIL — new methods don't exist.

- [ ] **Step 3: Add the transformation methods to `impl Inflections` in `inflections.rs`**

```rust
/// `post_comment` → `PostComment` (or `APIClient` when "API" is a registered acronym).
pub fn camelize(&self, s: &str) -> String {
    s.split('_')
        .map(|word| {
            let up = word.to_uppercase();
            if self.acronyms.contains(&up) {
                up
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + chars.as_str()
                    }
                }
            }
        })
        .collect()
}

/// `post_comment` → `postComment`.
pub fn camelize_lower(&self, s: &str) -> String {
    let mut parts = s.splitn(2, '_');
    let head = parts.next().unwrap_or("");
    let tail = parts.next().unwrap_or("");

    let head_lower = head.to_lowercase();
    if tail.is_empty() {
        head_lower
    } else {
        head_lower + &self.camelize(&format!("x_{tail}")[2..]) // camelize the tail
    }
}

/// `PostComment` → `post_comment`. Handles runs of uppercase (acronyms) correctly.
pub fn underscore(&self, s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() && i > 0 {
            let prev = chars[i - 1];
            let next = chars.get(i + 1);
            let next_is_lower = next.map_or(false, |n| n.is_lowercase());
            let prev_is_lower = prev.is_lowercase() || prev.is_numeric();
            if prev_is_lower || (prev.is_uppercase() && next_is_lower) {
                out.push('_');
            }
        }
        out.extend(c.to_lowercase());
    }
    out.replace('-', "_")
}

/// `post_comment` → `post-comment`.
pub fn dasherize(&self, s: &str) -> String {
    s.replace('_', "-")
}

/// `post_comment` → `Post comment`. Strips `_id` suffix.
pub fn humanize(&self, s: &str) -> String {
    let s = s.strip_suffix("_id").unwrap_or(s);
    let s = s.replace('_', " ");
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// `post_comment` → `POST_COMMENT`, `active-record` → `ACTIVE_RECORD`.
pub fn constantize(&self, s: &str) -> String {
    s.replace('-', "_").to_uppercase()
}
```

Note on `camelize_lower`: the implementation above has a subtle issue with the tail construction. Replace it with this cleaner version:

```rust
pub fn camelize_lower(&self, s: &str) -> String {
    let camelized = self.camelize(s);
    let mut chars = camelized.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p doido-core --test inflector_test`
Expected: PASS — all prior tests plus 9 new ones.

- [ ] **Step 5: Commit**

```bash
git add doido-core/src/inflector/inflections.rs doido-core/tests/inflector_test.rs
git commit -m "feat(core): add camelize, underscore, dasherize, humanize, constantize"
```

---

### Task 8: Compound Transforms + Acronym Support

**Files:**
- Modify: `doido-core/src/inflector/inflections.rs`
- Modify: `doido-core/tests/inflector_test.rs`

- [ ] **Step 1: Append these tests to `doido-core/tests/inflector_test.rs`**

```rust
// ── tableize and classify (inverses) ─────────────────────────────────────

#[test]
fn test_tableize() {
    let i = Inflections::default();
    assert_eq!(i.tableize("PostComment"), "post_comments");
    assert_eq!(i.tableize("Person"), "people");
}

#[test]
fn test_classify() {
    let i = Inflections::default();
    assert_eq!(i.classify("post_comments"), "PostComment");
}

#[test]
fn test_tableize_classify_are_inverses() {
    let i = Inflections::default();
    let original = "PostComment";
    assert_eq!(i.classify(&i.tableize(original)), original);
}

// ── foreign_key ───────────────────────────────────────────────────────────

#[test]
fn test_foreign_key() {
    let i = Inflections::default();
    assert_eq!(i.foreign_key("PostComment"), "post_comment_id");
    assert_eq!(i.foreign_key("Person"), "person_id");
}

// ── acronym support in camelize ───────────────────────────────────────────

#[test]
fn test_camelize_with_acronym() {
    let mut i = Inflections::default();
    i.acronym("API");
    i.acronym("HTML");
    assert_eq!(i.camelize("api_client"), "APIClient");
    assert_eq!(i.camelize("html_parser"), "HTMLParser");
}

// ── acronym support in underscore ────────────────────────────────────────

#[test]
fn test_underscore_acronym_sequence() {
    let i = Inflections::default();
    // No acronym registration needed — the algorithm handles uppercase runs.
    assert_eq!(i.underscore("APIClient"), "api_client");
    assert_eq!(i.underscore("HTMLParser"), "html_parser");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p doido-core --test inflector_test`
Expected: FAIL — new methods don't exist yet.

- [ ] **Step 3: Add `tableize`, `classify`, `foreign_key` to `impl Inflections`**

```rust
/// `PostComment` → `post_comments`  (underscore then pluralize).
pub fn tableize(&self, s: &str) -> String {
    self.pluralize(&self.underscore(s))
}

/// `post_comments` → `PostComment`  (singularize then camelize).
pub fn classify(&self, s: &str) -> String {
    self.camelize(&self.singularize(s))
}

/// `PostComment` → `post_comment_id`.
pub fn foreign_key(&self, s: &str) -> String {
    format!("{}_id", self.underscore(s))
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p doido-core --test inflector_test`
Expected: PASS — all tests including the 6 new ones.

- [ ] **Step 5: Commit**

```bash
git add doido-core/src/inflector/inflections.rs doido-core/tests/inflector_test.rs
git commit -m "feat(core): add tableize, classify, foreign_key, and acronym support"
```

---

### Task 9: Global Inflector Facade

**Files:**
- Modify: `doido-core/src/inflector/mod.rs`
- Modify: `doido-core/tests/inflector_test.rs`

- [ ] **Step 1: Append these tests to `doido-core/tests/inflector_test.rs`**

```rust
// ── static Inflector facade ───────────────────────────────────────────────

use doido_core::inflector::Inflector;

#[test]
fn test_inflector_static_pluralize() {
    assert_eq!(Inflector::pluralize("post"), "posts");
    assert_eq!(Inflector::pluralize("person"), "people");
    assert_eq!(Inflector::pluralize("sheep"), "sheep");
}

#[test]
fn test_inflector_static_singularize() {
    assert_eq!(Inflector::singularize("posts"), "post");
    assert_eq!(Inflector::singularize("people"), "person");
}

#[test]
fn test_inflector_static_camelize() {
    assert_eq!(Inflector::camelize("post_comment"), "PostComment");
}

#[test]
fn test_inflector_static_underscore() {
    assert_eq!(Inflector::underscore("PostComment"), "post_comment");
}

#[test]
fn test_inflector_static_tableize() {
    assert_eq!(Inflector::tableize("PostComment"), "post_comments");
}

#[test]
fn test_inflector_static_classify() {
    assert_eq!(Inflector::classify("post_comments"), "PostComment");
}

#[test]
fn test_inflector_static_foreign_key() {
    assert_eq!(Inflector::foreign_key("PostComment"), "post_comment_id");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p doido-core --test inflector_test`
Expected: FAIL — `Inflector` struct not yet defined.

- [ ] **Step 3: Replace `doido-core/src/inflector/mod.rs` with the full implementation**

```rust
pub mod inflections;
pub(crate) mod rules;

pub use inflections::Inflections;

use std::sync::OnceLock;

static INFLECTIONS: OnceLock<Inflections> = OnceLock::new();

/// Call this once at application boot, before any `Inflector::*` call.
/// The closure receives the default English rules; add custom overrides there.
///
/// ```rust
/// doido_core::inflector::init_inflections(|i| {
///     i.irregular("goose", "geese");
///     i.uncountable("bitcoin");
/// });
/// ```
pub fn init_inflections<F: FnOnce(&mut Inflections)>(configure: F) {
    let mut base = Inflections::default();
    configure(&mut base);
    // Silently ignore if already initialised (e.g. called twice in tests).
    let _ = INFLECTIONS.set(base);
}

fn global() -> &'static Inflections {
    INFLECTIONS.get_or_init(Inflections::default)
}

/// Static facade over the application-global `Inflections`.
/// All methods delegate to the global instance initialised by `init_inflections`
/// (or default English rules if `init_inflections` was never called).
pub struct Inflector;

impl Inflector {
    pub fn pluralize(s: &str) -> String   { global().pluralize(s) }
    pub fn singularize(s: &str) -> String { global().singularize(s) }
    pub fn camelize(s: &str) -> String    { global().camelize(s) }
    pub fn camelize_lower(s: &str) -> String { global().camelize_lower(s) }
    pub fn underscore(s: &str) -> String  { global().underscore(s) }
    pub fn dasherize(s: &str) -> String   { global().dasherize(s) }
    pub fn humanize(s: &str) -> String    { global().humanize(s) }
    pub fn tableize(s: &str) -> String    { global().tableize(s) }
    pub fn classify(s: &str) -> String    { global().classify(s) }
    pub fn foreign_key(s: &str) -> String { global().foreign_key(s) }
    pub fn constantize(s: &str) -> String { global().constantize(s) }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p doido-core --test inflector_test`
Expected: PASS — all tests including 7 new facade tests.

- [ ] **Step 5: Run the complete test suite**

Run: `cargo test -p doido-core`
Expected: PASS — all tests across all files.

- [ ] **Step 6: Commit**

```bash
git add doido-core/src/inflector/mod.rs doido-core/tests/inflector_test.rs
git commit -m "feat(core): add Inflector static facade backed by OnceLock global"
```

---

### Task 10: Final Wiring and Verification

**Files:**
- Modify: `doido-core/src/lib.rs`

- [ ] **Step 1: Ensure all public items are exported from `lib.rs`**

Replace `doido-core/src/lib.rs` with:

```rust
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
```

- [ ] **Step 2: Run the full test suite**

Run: `cargo test -p doido-core`
Expected: PASS — all tests green, zero warnings about unused items.

- [ ] **Step 3: Check for compiler warnings**

Run: `cargo build -p doido-core 2>&1 | grep warning`
Expected: No warnings (or only acceptable ones about unused imports in stub files if any).

- [ ] **Step 4: Commit**

```bash
git add doido-core/src/lib.rs
git commit -m "feat(core): finalize public API and re-exports"
```

---

## Self-Review

### Spec Coverage

| Spec requirement | Covered by |
|---|---|
| `Result<T>` alias using `anyhow` | Task 2 |
| Re-export `anyhow`, `thiserror` | Task 2, Task 10 |
| Re-export `async_trait`, `serde`, `tracing` | Task 10 |
| `Inflector::pluralize/singularize` | Task 6 |
| `Inflector::camelize/camelize_lower` | Task 7 |
| `Inflector::underscore/dasherize/humanize` | Task 7 |
| `Inflector::tableize/classify/foreign_key/constantize` | Task 8 |
| Acronym support in camelize/underscore | Task 8 |
| `Inflections` user-facing config struct | Task 4 |
| `init_inflections` boot hook | Task 9 |
| Default English rules (irregulars, uncountables) | Task 5 |
| Custom overrides take precedence | Task 6 (test_custom_*) |
| `tableize`/`classify` are inverses | Task 8 |
| `foreign_key` convention | Task 8 |
| Tracing helpers: `request`, `job`, `query`, `mail` | Task 3 |
| Leaf dependency (no workspace deps) | Task 1 — Cargo.toml |

### Placeholder Scan

No TBDs or TODOs remain — every step has complete, executable code.

### Type Consistency

- `Inflections` is the struct holding rules and transformation methods throughout.
- `Inflector` is the static facade delegating to the global `Inflections`.
- `init_inflections` is the one public function to configure the global.
- `pluralize`/`singularize`/`camelize` etc. are methods on `Inflections` AND static methods on `Inflector` — names are identical in both.
