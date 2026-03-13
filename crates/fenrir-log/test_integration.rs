use fenrir_log::{FenrirLogger, LogConfig};

fn main() -> anyhow::Result<()> {
    // Test 1: Basic tracing only
    println!("Test 1: Basic tracing logger");
    let logger1 = FenrirLogger::builder()
        .with_tracing()
        .without_perf_backend()
        .build()?;
    
    println!("- Has tracing: {}", logger1.has_tracing());
    println!("- Has perf backend: {}", logger1.has_perf_backend());
    
    // Test 2: With performance backend
    println!("\nTest 2: Hybrid logger");
    let logger2 = FenrirLogger::builder()
        .with_tracing()
        .with_perf_backend()
        .build()?;
    
    println!("- Has tracing: {}", logger2.has_tracing());
    println!("- Has perf backend: {}", logger2.has_perf_backend());
    
    // Test 3: Performance logging
    println!("\nTest 3: Performance logging");
    logger2.perf_log("test::module", "This is a performance log message");
    
    // Test 4: With rolling files
    println!("\nTest 4: Rolling file logger");
    let logger3 = FenrirLogger::builder()
        .with_tracing()
        .with_perf_backend()
        .with_rolling_file("/tmp/fenrir-test", "test.log", 1, 3) // 1MB files, keep 3
        .build()?;
    
    println!("- Has tracing: {}", logger3.has_tracing());
    println!("- Has perf backend: {}", logger3.has_perf_backend());
    
    // Test 5: Pod type logging
    #[cfg(feature = "perf")]
    {
        use fenrir_log::NetworkRequest;
        
        println!("\nTest 5: Pod type logging");
        let request = NetworkRequest {
            url: "https://example.com",
            method: "GET",
            duration_ms: 42,
            status_code: 200,
        };
        
        logger3.perf_log_pod("network", &request, "Request completed");
    }
    
    println!("\nAll tests passed!");
    Ok(())
}
