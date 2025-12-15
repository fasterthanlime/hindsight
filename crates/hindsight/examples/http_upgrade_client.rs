use hindsight::Tracer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Hindsight server via HTTP Upgrade
    println!("🌐 Connecting to Hindsight server at 127.0.0.1:1990 via HTTP Upgrade...");
    let tracer = Tracer::connect_http("127.0.0.1:1990").await?;
    println!("✅ Connected! HTTP upgraded to Rapace protocol!");

    // Send 10 test spans
    println!("📊 Sending 10 test spans...");
    for i in 0..10 {
        let span = tracer
            .span("http_upgrade_test")
            .with_attribute("iteration", i)
            .with_attribute("transport", "http_upgrade")
            .start();

        // Simulate some work
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        span.end();
        println!("  ✓ Sent span {}/10", i + 1);
    }

    // Wait for batching to complete
    println!("⏳ Waiting for batching to complete...");
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    println!("🎉 Done! HTTP Upgrade working perfectly!");
    Ok(())
}
