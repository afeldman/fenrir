# Fenrir-Log

Hybrid logging library for the Fenrir browser, combining the observability of `tracing`
with the ultra-low latency of `inqjet`.

## Features

- **Hybrid Architecture**: Use `tracing` for structured logs and `inqjet` for performance-critical paths
- **Feature Flags**: Enable only what you need (`tracing`, `rolling`, `perf`)
- **Zero-Cost Abstraction**: `Pod` type support for efficient structured logging
- **Runtime Configuration**: Adjust log levels and backends at runtime
- **Rolling File Support**: Built-in log rotation with compression

## Quick Start

```rust
use fenrir_log::{FenrirLogger, LogConfig};

// Basic setup with tracing only
let logger = FenrirLogger::builder()
    .with_tracing()
    .build()?;

// Full setup with both backends
let logger = FenrirLogger::builder()
    .with_tracing()
    .with_perf_backend()  // Enable inqjet for performance logs
    .with_rolling_file("logs/fenrir.log", 10 * 1024 * 1024, 7) // 10MB files, keep 7
    .build()?;

// Performance-critical logging (uses inqjet if enabled)
logger.perf_log("network::request", format!("Request to {}", url));

// Standard logging (uses tracing)
tracing::info!("User logged in: {}", username);
```

## Configuration

Add to your `Cargo.toml`:

```toml
[dependencies]
fenrir-log = { path = "../crates/fenrir-log", features = ["tracing", "perf", "rolling"] }
```

## Benchmarks

Run with: `cargo bench --features perf`

## License

EUPL-1.2
