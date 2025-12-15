//! UI components for Hindsight

use sycamore::prelude::*;
use hindsight_protocol::*;

use crate::navigation::NavigationState;
use crate::routing::Route;

/// TraceCard component - displays a single trace summary
#[component]
pub fn TraceCard(TraceCardProps { trace, nav_state }: TraceCardProps) -> View {
    let duration_text = trace
        .duration_nanos
        .map(|nanos| format!("{:.2}ms", nanos as f64 / 1_000_000.0))
        .unwrap_or_else(|| "—".to_string());

    let type_class = format!("type-{}", trace.trace_type.to_string().to_lowercase());

    let trace_id = trace.trace_id.clone();
    let on_click = move |_| {
        nav_state.navigate_to(Route::TraceDetail {
            trace_id: trace_id.clone(),
        });
    };

    view! {
        div(class="trace-item", on:click=on_click, style="cursor: pointer;") {
            div(class="trace-name") { (trace.root_span_name.clone()) }
            div(class="trace-duration") { (duration_text) }
            div(class="trace-meta") {
                div(class="trace-meta-item") {
                    span(class="trace-meta-label") { "svc" }
                    span { (trace.service_name.clone()) }
                }
                div(class="trace-meta-item") {
                    span(class="trace-meta-label") { "spans" }
                    span { (trace.span_count.to_string()) }
                }
                (if trace.has_errors {
                    view! {
                        div(class="trace-meta-item trace-error-indicator") {
                            "errors"
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
    pub nav_state: NavigationState,
}
