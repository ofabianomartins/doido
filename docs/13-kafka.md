# doido-kafka — Spec

No direct Rails analogue. Optional, standalone crate.

## Decisions (resolved in interview)

- **Kafka-specific crate** — no generic messaging abstraction; Kafka concepts are first-class
- **Opt-in** — not part of the default framework bundle; added explicitly to `Cargo.toml`
- **Kafka client:** `rskafka` (pure Rust, zero native deps)
- **Consumer style:** `#[consumer]` macro — single-topic shorthand OR multi-topic with `#[topic]` per method (option C)
- **Serialization:** pluggable `MessageCodec` trait, `JsonCodec` default
- **Error handling:** every message is dispatched as a `doido-jobs` job — retries, backoff, and dead letter queue all handled by the job system

## Core Design: Consumer as Job Dispatcher

Kafka consumers are thin dispatchers. They deserialize the message and enqueue a job. All reliability concerns (retry, exponential backoff, dead letter) are handled by `doido-jobs`.

```
Kafka topic → Consumer (deserialize) → enqueue Job → Worker processes Job
                                                          ├─ Ok  → ack offset
                                                          └─ Err → retry / dead letter (doido-jobs)
```

Offset is committed **only after the job is successfully enqueued**, not after processing. Processing is fully async via the job queue.

## Consumer Definition

### Single-topic (simple)

```rust
#[consumer(topic = "orders.created", group = "billing")]
struct OrderCreatedConsumer;

impl OrderCreatedConsumer {
    async fn handle(ctx: &ConsumerContext, msg: OrderCreated) -> Result<()> {
        ProcessOrderJob { order_id: msg.order_id }.enqueue().await
    }
}
```

### Multi-topic (related event family)

```rust
#[consumer(group = "billing")]
struct OrderConsumer;

impl OrderConsumer {
    #[topic("orders.created")]
    async fn on_created(ctx: &ConsumerContext, msg: OrderCreated) -> Result<()> {
        ProcessOrderJob { order_id: msg.order_id }.enqueue().await
    }

    #[topic("orders.cancelled")]
    async fn on_cancelled(ctx: &ConsumerContext, msg: OrderCancelled) -> Result<()> {
        CancelBillingJob { order_id: msg.order_id }.enqueue().await
    }
}
```

## `#[consumer]` Macro Attributes

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `topic` | `&str` | — | Kafka topic (single-topic form) |
| `group` | `&str` | app name | Consumer group ID |
| `partitions` | `u32` | all | Number of partitions to consume |
| `offset` | `&str` | `"latest"` | `"earliest"` \| `"latest"` |
| `codec` | `&str` | `"json"` | `"json"` \| custom registered codec |

## `ConsumerContext`

```rust
ctx.topic()              // topic name of current message
ctx.partition()          // partition number
ctx.offset()             // message offset
ctx.key()                // raw message key bytes
ctx.headers()            // Kafka message headers
ctx.db                   // database connection handle
ctx.kafka                // KafkaHandle — can produce from within a consumer
```

## Producer API

Producers injectable anywhere via `ctx.kafka` or globally via `Kafka::produce`.

```rust
// In a controller
ctx.kafka.produce("orders.created", OrderCreated { order_id: 42 }).await?;

// With explicit key (for partition routing)
ctx.kafka.produce_keyed("orders.created", &user_id.to_string(), msg).await?;

// With headers
ctx.kafka.produce_with_headers("orders.created", msg, headers).await?;

// Global shorthand (callable from jobs, mailers, anywhere)
Kafka::produce("orders.created", msg).await?;
```

## `MessageCodec` Trait (pluggable serialization)

```rust
pub trait MessageCodec: Send + Sync {
    fn encode<T: Serialize>(&self, value: &T) -> Result<Vec<u8>>;
    fn decode<T: DeserializeOwned>(&self, bytes: &[u8]) -> Result<T>;
    fn content_type(&self) -> &str;
}
```

Built-in codecs:

| Codec | Feature flag | Description |
|-------|-------------|-------------|
| `JsonCodec` | default | `serde_json`, human-readable |
| `MsgpackCodec` | `feature = "codec-msgpack"` | `rmp-serde`, compact binary |

Custom codec registered at boot:

```rust
doido_kafka::codecs().register("protobuf", Box::new(MyProtobufCodec));
```

Codec selected per consumer via `#[consumer(codec = "json")]` or per-produce call.

## Consumer Registry

Consumers registered at app boot, similar to channels:

```rust
// In app initializer
doido_kafka::consumers().register(Box::new(OrderCreatedConsumer));
doido_kafka::consumers().register(Box::new(OrderConsumer));
```

Or via a macro in the app entrypoint:

```rust
kafka_consumers! {
    OrderCreatedConsumer,
    OrderConsumer,
    PaymentConsumer,
}
```

## Config

```toml
[kafka]
brokers = ["localhost:9092"]   # list of bootstrap brokers
client_id = "myapp"

[kafka.consumer]
offset = "latest"              # default offset reset
codec = "json"                 # default codec

[kafka.producer]
acks = "all"                   # "none" | "leader" | "all"
compression = "lz4"            # "none" | "gzip" | "snappy" | "lz4"
codec = "json"
```

## Generator

Added to `doido-generators`:

```
doido generate consumer Order created cancelled
```

Creates:
- `consumers/order_consumer.rs` — multi-topic `#[consumer]` struct with one `#[topic]` method per event name
- Corresponding job stubs if they don't exist

Prints a hint:
```
Register in your app initializer:
    doido_kafka::consumers().register(Box::new(OrderConsumer));
```

## Module Structure

```
doido-kafka/
  src/
    lib.rs
    consumer/
      mod.rs            ← Consumer trait + ConsumerContext
      registry.rs       ← ConsumerRegistry
      runner.rs         ← async task: poll → deserialize → dispatch job
    producer/
      mod.rs            ← KafkaHandle + Kafka::produce global
    codec/
      mod.rs            ← MessageCodec trait + codec registry
      json.rs
      msgpack.rs
    config.rs           ← KafkaConfig deserialized from doido-config
```

## Known Requirements

- `doido-kafka` is an **optional workspace crate** — not imported by `doido` by default
- `rskafka` as the sole Kafka client; no librdkafka dependency
- Every `handle` call enqueues a `doido-jobs` job — offset committed after enqueue, not after job completion
- `MessageCodec` trait is the only serialization abstraction; `JsonCodec` ships as default
- `ConsumerRegistry` maps topic strings to consumer impls
- Producers available globally via `Kafka::produce` and via `ctx.kafka` in controllers/jobs/channels
- Consumer tasks started by `doido server` and `doido worker` commands

## TDD Surface

- Test `#[consumer]` macro registers correct topic(s) in registry
- Test `handle` receives correctly deserialized message from raw bytes
- Test `#[topic]` methods routed by topic name in multi-topic consumer
- Test `handle` returning `Ok` commits offset
- Test `handle` returning `Err` does NOT commit offset
- Test dispatched job is enqueued in `doido-jobs` queue
- Test `JsonCodec` encodes and decodes typed structs correctly
- Test custom `MessageCodec` registered and used when specified
- Test `Kafka::produce` serializes and sends to correct topic
- Test `produce_keyed` routes to correct partition via key
- Integration test: produce message → consumer receives → job enqueued → drain jobs → side effect observed
