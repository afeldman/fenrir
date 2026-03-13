//! Benchmarks for Fenrir-Log performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fenrir_log::{FenrirLogger, LogConfig};
use std::time::Instant;

fn bench_tracing_only(c: &mut Criterion) {
    c.bench_function("tracing_only_info", |b| {
        let logger = FenrirLogger::builder()
            .with_tracing()
            .without_perf_backend()
            .build()
            .unwrap();
        
        b.iter(|| {
            black_box(tracing::info!("Benchmark message"));
        });
    });
}

#[cfg(feature = "perf")]
fn bench_perf_only(c: &mut Criterion) {
    c.bench_function("perf_only_info", |b| {
        let logger = FenrirLogger::builder()
            .without_tracing()
            .with_perf_backend()
            .build()
            .unwrap();
        
        b.iter(|| {
            black_box(logger.perf_log("bench", "Benchmark message"));
        });
    });
}

#[cfg(feature = "perf")]
fn bench_hybrid(c: &mut Criterion) {
    c.bench_function("hybrid_info", |b| {
        let logger = FenrirLogger::builder()
            .with_tracing()
            .with_perf_backend()
            .build()
            .unwrap();
        
        b.iter(|| {
            black_box(logger.perf_log("bench", "Benchmark message"));
        });
    });
}

fn bench_pod_types(c: &mut Criterion) {
    #[cfg(feature = "perf")]
    {
        use fenrir_log::NetworkRequest;
        
        c.bench_function("pod_logging", |b| {
            let logger = FenrirLogger::builder()
                .with_tracing()
                .with_perf_backend()
                .build()
                .unwrap();
            
            let request = NetworkRequest {
                url: "https://example.com".to_string(),
                method: "GET".to_string(),
                duration_ms: 42,
                status_code: 200,
            };
            
            b.iter(|| {
                black_box(logger.perf_log_pod("network", &request, "Request completed"));
            });
        });
    }
}

fn bench_concurrent_logging(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent");
    group.sample_size(10);
    
    group.bench_function("tracing_10_threads", |b| {
        let logger = FenrirLogger::builder()
            .with_tracing()
            .without_perf_backend()
            .build()
            .unwrap();
        
        b.iter_custom(|iters| {
            let start = Instant::now();
            
            std::thread::scope(|s| {
                for _ in 0..10 {
                    s.spawn(|| {
                        for _ in 0..(iters / 10) {
                            tracing::info!("Thread message");
                        }
                    });
                }
            });
            
            start.elapsed()
        });
    });
    
    #[cfg(feature = "perf")]
    group.bench_function("perf_10_threads", |b| {
        let logger = FenrirLogger::builder()
            .without_tracing()
            .with_perf_backend()
            .build()
            .unwrap();
        
        b.iter_custom(|iters| {
            let start = Instant::now();
            
            std::thread::scope(|s| {
                for i in 0..10 {
                    let logger_ref = &logger;
                    s.spawn(move || {
                        for j in 0..(iters / 10) {
                            logger_ref.perf_log("bench", format!("Thread {}: Message {}", i, j));
                        }
                    });
                }
            });
            
            start.elapsed()
        });
    });
}

#[cfg(feature = "perf")]
criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = 
        bench_tracing_only,
        bench_perf_only,
        bench_hybrid,
        bench_pod_types,
        bench_concurrent_logging
);

#[cfg(not(feature = "perf"))]
criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = 
        bench_tracing_only,
        bench_concurrent_logging
);

criterion_main!(benches);
