//! Client library for sending spans to Hindsight tracing server.
//!
//! # Example
//!
//! ```no_run
//! use hindsight::Tracer;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect via HTTP upgrade
//!     let tracer = Tracer::connect_http("localhost:9090").await?;
//!
//!     let span = tracer.span("my_operation")
//!         .with_attribute("user_id", 123)
//!         .with_attribute("endpoint", "/api/users")
//!         .start();
//!
//!     // Do work...
//!
//!     span.end();
//!     Ok(())
//! }
//! ```

mod span_builder;
mod tracer;

pub use hindsight_protocol::*;
pub use span_builder::{ActiveSpan, IntoAttributeValue, SpanBuilder};
pub use tracer::{Tracer, TracerError};
