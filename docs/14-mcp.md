# doido-mcp — Spec

Analogue: Model Context Protocol (MCP) — Anthropic's open protocol for AI ↔ app integration.
Optional, standalone crate — not part of the default framework bundle.

## Decisions (resolved in interview)

- **Transport:** HTTP + SSE (Streamable HTTP), mounted at configurable path (default `/mcp`)
- **Tool definition:** `#[tool]` macro on plain async functions
- **Resources:** both `#[resource]` on functions AND `#[mcp_resource]` on sea-orm models; content always `text`
- **MCP Client:** both raw dynamic calls (`call_tool` / `read_resource`) and typed generated wrappers via `doido generate mcp_client`
- **Auth:** Tower middleware as default; OAuth 2.1 as opt-in (`feature = "mcp-oauth"`)
- **Generator:** `doido generate mcp_tool`, `doido generate mcp_resource`, `doido generate mcp_client`

## MCP Server — Tools

Tools are callable functions AI models can invoke by name. Defined with `#[tool]` on async functions:

```rust
use doido_mcp::prelude::*;

#[tool(description = "Search blog posts by keyword")]
async fn search_posts(ctx: &ToolContext, input: SearchPostsInput) -> Result<ToolOutput> {
    let posts = Post::find()
        .filter(Column::Title.contains(&input.keyword))
        .all(&ctx.db)
        .await?;
    ToolOutput::text(serde_json::to_string_pretty(&posts)?)
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SearchPostsInput {
    /// The keyword to search for in post titles
    keyword: String,
    /// Maximum number of results (default 10)
    limit: Option<u32>,
}
```

`#[tool]` macro responsibilities:
- Registers the function in the global `ToolRegistry` under its function name
- Auto-generates MCP JSON Schema for `input` via `schemars` (field docs → descriptions)
- Wraps the function signature to match MCP JSON-RPC dispatch

## MCP Server — Resources

Resources are addressable data sources AI can list and read by URI.

### Manual `#[resource]` on functions

```rust
#[resource(
    uri      = "posts://{id}",
    name     = "blog_post",
    description = "A single blog post by ID"
)]
async fn post_resource(ctx: &ResourceContext, id: i64) -> Result<ResourceContent> {
    let post = Post::find_by_id(id).one(&ctx.db).await?
        .ok_or(McpError::NotFound)?;
    ResourceContent::text(serde_json::to_string_pretty(&post)?)
}

#[resource(uri = "posts://", name = "blog_posts", description = "All blog posts")]
async fn posts_list_resource(ctx: &ResourceContext) -> Result<ResourceContent> {
    let posts = Post::find().all(&ctx.db).await?;
    ResourceContent::text(serde_json::to_string_pretty(&posts)?)
}
```

### Auto-expose sea-orm models with `#[mcp_resource]`

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, McpResource)]
#[sea_orm(table_name = "posts")]
#[mcp_resource(uri = "posts", description = "Blog posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub body: String,
}
```

`#[mcp_resource]` auto-generates two resources:
- `posts://` — list all records as text
- `posts://{id}` — single record by primary key as text

Both return `ResourceContent::text(serde_json::to_string_pretty(&record)?)`.

## `ToolContext` and `ResourceContext`

```rust
ctx.db          // DatabaseConnection — same as controllers/jobs
ctx.config      // Arc<Config>
ctx.cache       // CacheHandle
ctx.meta        // caller metadata: client_id, request_id
```

## MCP Server — Registration

Tools and resources registered at app boot via `mcp_server!` block inside `routes!`:

```rust
routes! {
    middleware!(ApiKeyAuth);    // auth middleware protects the endpoint

    mcp_server! {
        tools: [search_posts, create_post, delete_post],
        resources: [post_resource, posts_list_resource],
        // #[mcp_resource] models auto-registered — no need to list them
    }

    resources!(posts, PostsController);
}
```

MCP endpoint mounted at path configured in `doido.toml` (default `/mcp`).

## MCP Server — HTTP + SSE Transport

Follows the MCP Streamable HTTP spec:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `POST /mcp` | POST | JSON-RPC request (initialize, tools/call, resources/read, etc.) |
| `GET /mcp` | GET | SSE stream for server-initiated messages and progress |

Content-Type for SSE: `text/event-stream`.
Each JSON-RPC response sent as an SSE `data:` event.

## MCP Client — Raw Dynamic Calls

For dynamic or unknown servers — raw JSON in, raw JSON out:

```rust
// In a controller, job, or anywhere ctx is available
let result = ctx.mcp
    .server("analytics")
    .call_tool("top_posts", json!({ "limit": 10 }))
    .await?;

let content = ctx.mcp
    .server("filesystem")
    .read_resource("file:///data/report.txt")
    .await?;

// List available tools on a connected server
let tools = ctx.mcp.server("analytics").list_tools().await?;
```

## MCP Client — Typed Generated Wrappers

For known servers — generate a typed Rust client once, get compile-time safety:

```
doido generate mcp_client http://analytics-service/mcp --name AnalyticsClient
```

Introspects the live server's tool list → generates `clients/analytics_client.rs`:

```rust
// Generated — do not edit manually
pub struct AnalyticsClient<'ctx> { /* ... */ }

impl<'ctx> AnalyticsClient<'ctx> {
    /// Top N posts by view count
    pub async fn top_posts(&self, input: TopPostsInput) -> Result<String> { ... }

    /// Daily view statistics for a date range
    pub async fn daily_views(&self, input: DailyViewsInput) -> Result<String> { ... }
}

#[derive(Serialize)]
pub struct TopPostsInput { pub limit: u32 }

#[derive(Serialize)]
pub struct DailyViewsInput { pub from: String, pub to: String }
```

Usage in app code:

```rust
let client = ctx.mcp.server::<AnalyticsClient>("analytics");
let posts = client.top_posts(TopPostsInput { limit: 10 }).await?;
```

Re-run `doido generate mcp_client` to refresh if the server schema changes.

## Authentication

### Default — Tower middleware (no extra config)

The `mcp_server!` block sits inside `routes!`, inheriting any middleware above it:

```rust
routes! {
    namespace!("/api", {
        middleware!(BearerTokenAuth);   // covers everything below
        mcp_server! { tools: [...] }
        resources!(posts, PostsController);
    });
}
```

### OAuth 2.1 — opt-in feature flag

```toml
# Cargo.toml
doido-mcp = { path = "../doido-mcp", features = ["mcp-oauth"] }
```

```toml
# config/doido.toml
[mcp.server.auth]
strategy = "oauth"
issuer   = "https://myapp.com"
```

When enabled, `doido-mcp` serves the MCP OAuth 2.1 discovery and token endpoints:
- `GET /.well-known/oauth-authorization-server`
- `POST /oauth/token`

Integrates with `doido-config` credentials for client secrets.

## Config

```toml
[mcp.server]
mount       = "/mcp"           # HTTP endpoint path
ping_interval = 30             # SSE keepalive ping seconds

[[mcp.client.servers]]
name = "analytics"
url  = "http://analytics-service/mcp"

[[mcp.client.servers]]
name = "filesystem"
url  = "http://localhost:3001/mcp"
headers = { Authorization = "Bearer ${FS_MCP_TOKEN}" }
```

## Module Structure

```
doido-mcp/
  src/
    lib.rs
    server/
      mod.rs            ← MCP server: request routing, SSE handler
      registry.rs       ← ToolRegistry + ResourceRegistry
      tool.rs           ← #[tool] macro support + ToolContext
      resource.rs       ← #[resource] + #[mcp_resource] macro support
      protocol.rs       ← JSON-RPC 2.0 encode/decode per MCP spec
      transport/
        http_sse.rs     ← axum handlers for POST /mcp + GET /mcp SSE
    client/
      mod.rs            ← McpClient trait + raw call_tool/read_resource
      typed.rs          ← typed wrapper base for generated clients
      pool.rs           ← connection pool per named server
    auth/
      middleware.rs     ← Tower layer integration
      oauth.rs          ← OAuth 2.1 endpoints (feature = "mcp-oauth")
    config.rs           ← McpConfig deserialized from doido-config
```

## Generator Additions to `doido-generators`

| Command | Files Created |
|---------|--------------|
| `doido generate mcp_tool search_posts` | `mcp/tools/search_posts.rs` with `#[tool]` stub |
| `doido generate mcp_resource post` | `mcp/resources/post_resource.rs` with `#[resource]` stub |
| `doido generate mcp_client <url> --name <Name>` | `clients/<name>_client.rs` — typed wrapper from live server |

## Known Requirements

- Optional crate — not imported by default `doido` bundle
- HTTP + SSE transport built on axum handlers
- `#[tool]` and `#[resource]` macros auto-generate MCP JSON Schema via `schemars`
- `#[mcp_resource]` on sea-orm models auto-generates list + single-record resources
- All resources return `ResourceContent::text(...)` — no binary/blob support in v1
- `ToolRegistry` and `ResourceRegistry` populated at boot; `mcp_server!` macro declares what to expose
- MCP client connection pool: one persistent SSE connection per named server
- OAuth 2.1 only available via `feature = "mcp-oauth"`

## TDD Surface

- Test `#[tool]` macro registers function in `ToolRegistry` with correct name and schema
- Test `#[resource]` registers URI pattern and handler in `ResourceRegistry`
- Test `#[mcp_resource]` on a model generates list and single-record resources
- Test JSON Schema generated from `SearchPostsInput` matches expected MCP shape
- Test `POST /mcp` with `tools/call` dispatches to correct tool function
- Test `POST /mcp` with `resources/read` dispatches to correct resource handler
- Test `GET /mcp` SSE stream sends `ping` events at configured interval
- Test unknown tool name returns MCP `MethodNotFound` JSON-RPC error
- Test MCP client `call_tool` sends correct JSON-RPC to server and parses response
- Test MCP client `read_resource` fetches and returns text content
- Test typed generated client methods serialize input and deserialize output correctly
- Test Tower middleware rejects unauthenticated `/mcp` requests
- Integration test: register tool → POST /mcp tools/call → tool executes with real db → response correct
- Integration test: `#[mcp_resource]` model → GET list resource → returns all records as text
