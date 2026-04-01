# Doido Framework — Overview

Doido is a Rails-inspired web framework written in Rust.
It targets developer productivity through conventions, a rich CLI,
and a modular architecture where each sub-crate can be used independently.

## Core Philosophy

- **Convention over configuration** — sensible defaults, opt-in overrides
- **Batteries included** — routing, ORM, views, mailer, jobs, cache out of the box
- **Modular** — each crate is independently usable and testable
- **TDD-first** — every module ships with test helpers and a well-defined test surface
- **Async-native** — Tokio + async/await throughout; no sync shims

## Technology Choices

| Concern        | Library       | Rails analogue         |
|----------------|---------------|------------------------|
| HTTP server    | axum          | Puma / Rack            |
| ORM            | sea-orm       | Active Record          |
| Async runtime  | tokio         | (implicit in Rails)    |
| Serialization  | serde         | (implicit in Rails)    |
| Tracing        | tracing       | Rails logger           |

## Workspace Crates

| Crate              | Rails Analogue        | Responsibility                              |
|--------------------|-----------------------|---------------------------------------------|
| `doido`            | `rails` binary        | Entry point, app scaffold                   |
| `doido-core`       | Active Support        | Shared traits, errors, utilities            |
| `doido-router`     | Action Dispatch       | Route DSL, URL helpers                      |
| `doido-controller` | Action Controller     | Request handling, params, responses         |
| `doido-model`      | Active Record         | Sea-ORM integration, model macros           |
| `doido-view`       | Action View           | Template rendering, response helpers        |
| `doido-config`     | Rails config/         | Environments, credentials, settings         |
| `doido-cli`        | `rails` CLI           | generate, migrate, server, console          |
| `doido-mailer`     | Action Mailer         | Email composition and delivery              |
| `doido-jobs`       | Active Job            | Async background job queue                  |
| `doido-cache`      | Active Support Cache  | Pluggable caching layer                     |
| `doido-middleware` | Rack middleware       | Logging, CORS, sessions, auth               |

## TDD Strategy

Each crate exposes a `doido_*::testing` module with:
- Test fixtures and factories
- In-memory fakes for external dependencies (DB, mailer, queue, cache)
- Integration helpers that wire up a minimal Doido application

Tests are organized as:
- **Unit** — pure logic, no I/O
- **Integration** — crate boundary tests with fakes
- **System** — full-stack HTTP tests using a real in-memory server
