//! UI components for Hindsight

use sycamore::prelude::*;
use hindsight_protocol::*;

/// TraceCard component - displays a single trace summary
#[component]
pub fn TraceCard(TraceCardProps { trace }: TraceCardProps) -> View {
    let duration_text = trace
        .duration_nanos
        .map(|nanos| format!("{:.2}ms", nanos as f64 / 1_000_000.0))
        .unwrap_or_else(|| "—".to_string());

    let type_class = format!("type-{}", trace.trace_type.to_string().to_lowercase());

    view! {
        div(class="trace-card") {
            div(class="trace-header") {
                div(class="trace-name") { (trace.root_span_name.clone()) }
                div(class="trace-duration") { (duration_text) }
            }
            div(class="trace-meta") {
                span { "🏷️ " (trace.service_name.clone()) }
                span { "📊 " (trace.span_count) " spans" }
                (if trace.has_errors {
                    view! {
                        span(style="color: #ef4444;") {
                            "⚠️ errors"
                        }
                    }
                } else {
                    view! {}
                })
                span(class=format!("trace-type-badge {}", type_class)) {
                    (trace.trace_type.to_string())
                }
            }
        }
    }
}

#[derive(Props)]
pub struct TraceCardProps {
    pub trace: TraceSummary,
}
