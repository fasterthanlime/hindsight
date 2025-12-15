//! Trace detail view component

use sycamore::prelude::*;
use hindsight_protocol::*;

use crate::navigation::NavigationState;
use crate::routing::Route;

/// Trace detail view - shows full trace information
#[component]
pub fn TraceDetail(props: TraceDetailProps) -> View {
    let nav_state = props.nav_state;
    let trace_id = props.trace_id;

    // Back button handler
    let on_back = move |_| {
        nav_state.navigate_to(Route::TraceList);
    };

    view! {
        div(class="trace-detail") {
            div(class="detail-header") {
                button(class="btn", on:click=on_back) { "← Back to Traces" }
                div(class="detail-title") {
                    "Trace Detail"
                }
                div(class="detail-meta") {
                    span(class="trace-meta-label") { "ID" }
                    " "
                    span(style="font-family: var(--font-mono); font-size: var(--text-xs);") {
                        (trace_id.to_hex())
                    }
                }
            }

            div(class="detail-content") {
                div(class="placeholder") {
                    p { "Full trace visualization coming soon..." }
                    p(style="margin-top: var(--space-4); color: var(--text-secondary);") {
                        "This will show:"
                    }
                    ul(style="margin-top: var(--space-2); padding-left: var(--space-6); color: var(--text-secondary);") {
                        li { "Waterfall chart of spans" }
                        li { "Span hierarchy tree" }
                        li { "Timing information" }
                        li { "Error details" }
                        li { "Tags and attributes" }
                    }
                }
            }
        }
    }
}

#[derive(Props)]
pub struct TraceDetailProps {
    pub trace_id: TraceId,
    pub nav_state: NavigationState,
}
