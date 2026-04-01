# doido-jobs — Spec

Rails analogue: **Active Job**

## Decisions (resolved in interview)

- **Queue backends: pluggable** — in-memory, database (sea-orm), and Redis selectable via config
- **Retry: automatic, exponential backoff by default**, configured per-job on the struct
- **Exhausted jobs written to a dead letter queue** — inspectable and re-enqueueable via CLI

## Job Definition

```rust
#[job(queue = "default", max_retries = 5, backoff = "exponential")]
struct SendWelcomeEmail {
    pub user_id: i64,
}

impl SendWelcomeEmail {
    async fn perform(&self, ctx: &JobContext) -> Result<()> {
        let user = User::find_by_id(self.user_id).one(&ctx.db).await?;
        UserMailer::welcome(&user).deliver_now().await?;
        Ok(())
    }
}
```

## `#[job(...)]` Macro Attributes

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `queue` | `&str` | `"default"` | Queue name |
| `max_retries` | `u32` | `3` | Max attempts before dead letter |
| `backoff` | `&str` | `"exponential"` | `"exponential"` \| `"linear"` \| `"none"` |
| `backoff_base` | `u64` (seconds) | `5` | Base delay for backoff calculation |
| `timeout` | `u64` (seconds) | `30` | Max seconds a single attempt may run |
| `priority` | `i32` | `0` | Higher = processed first |

Exponential backoff formula: `backoff_base * 2^(attempt - 1)` seconds.  
Attempt 1 → 5s, attempt 2 → 10s, attempt 3 → 20s, attempt 4 → 40s, attempt 5 → 80s.

## `JobQueue` Trait (pluggable)

```rust
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, job: &dyn JobPayload) -> Result<JobId>;
    async fn enqueue_at(&self, job: &dyn JobPayload, at: DateTime<Utc>) -> Result<JobId>;
    async fn dequeue(&self, queues: &[&str]) -> Result<Option<JobEnvelope>>;
    async fn ack(&self, id: JobId) -> Result<()>;
    async fn nack(&self, id: JobId, retry_at: Option<DateTime<Utc>>) -> Result<()>;
    async fn dead_letter(&self, id: JobId, reason: &str) -> Result<()>;
}
```

## Built-in Backends

| Backend | Feature flag | Description |
|---------|-------------|-------------|
| `InMemoryQueue` | always | `tokio::sync::Mutex<VecDeque>`, lost on restart, dev/test |
| `DbQueue` | `feature = "jobs-db"` | Jobs stored in `doido_jobs` sea-orm table |
| `RedisQueue` | `feature = "jobs-redis"` | Redis `LPUSH`/`BRPOP`, compatible with Sidekiq wire format |

Selected via config:

```toml
[jobs]
backend = "db"          # "memory" | "db" | "redis"
queues = ["default", "mailers", "critical"]
concurrency = 5         # worker threads per queue

[jobs.redis]
url = "${REDIS_URL}"
```

## Job Lifecycle

```
enqueue → [queue] → dequeue (worker) → perform()
                        ├─ Ok   → ack (done)
                        └─ Err  → nack
                                    ├─ retries < max_retries → re-enqueue with backoff delay
                                    └─ retries == max_retries → dead_letter queue
```

## Dead Letter Queue

- Failed jobs written to a separate `dead_letters` store (same backend)
- Each entry stores: job payload, last error message, attempt count, failed_at timestamp
- Inspectable and re-enqueueable via CLI:

```
doido jobs:failed             ← list dead letter jobs
doido jobs:retry <job_id>     ← re-enqueue a specific dead letter job
doido jobs:retry --all        ← re-enqueue all dead letter jobs
doido jobs:discard <job_id>   ← permanently remove from dead letter queue
```

## Enqueue API

```rust
// Enqueue immediately
SendWelcomeEmail { user_id: 42 }.enqueue().await?;

// Enqueue with delay
SendWelcomeEmail { user_id: 42 }.enqueue_in(Duration::from_secs(60)).await?;

// Enqueue at specific time
SendWelcomeEmail { user_id: 42 }.enqueue_at(scheduled_at).await?;

// Enqueue to specific queue
SendWelcomeEmail { user_id: 42 }.on_queue("critical").enqueue().await?;
```

## Test Helpers

```rust
use doido_jobs::testing::JobQueue as TestQueue;

// Assert a job was enqueued
TestQueue::assert_enqueued::<SendWelcomeEmail>(|job| job.user_id == 42);

// Drain and execute all enqueued jobs
TestQueue::drain(&ctx).await?;

// Assert dead letter queue
TestQueue::assert_dead_lettered::<SendWelcomeEmail>(1);
```

## Known Requirements

- `#[job(...)]` macro implements `JobPayload` trait and serializes to JSON
- Job structs must be `Serialize + Deserialize` (serde)
- `JobContext` carries `db: DatabaseConnection` and app config
- Worker process started via `doido worker` or `doido server` (embedded worker mode)
- In test env, `InMemoryQueue` always used; jobs do not auto-execute unless `drain()` called
- Dead letter entries never auto-deleted; require explicit CLI action

## TDD Surface

- Test `#[job]` macro generates correct `JobPayload` serialization
- Test `enqueue()` adds job to queue
- Test `enqueue_in()` schedules job at correct future time
- Test `perform()` executes successfully and job is acked
- Test failed `perform()` triggers nack and increments retry count
- Test exponential backoff delays are calculated correctly
- Test job moved to dead letter after `max_retries` exhausted
- Test `TestQueue::drain()` executes all enqueued jobs
- Test `TestQueue::assert_enqueued` passes and fails correctly
- Integration test: controller enqueues job → drain in test → side effect observed
