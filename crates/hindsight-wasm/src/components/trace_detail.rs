//! Trace detail view component

use hindsight_protocol::*;
use rapace::RpcSession;
use std::collections::HashMap;
use std::sync::Arc;
use sycamore::prelude::*;

use crate::navigation::NavigationState;
use crate::routing::Route;

/// Hierarchical span node for tree rendering
#[derive(Clone, Debug)]
struct SpanNode {
    span: Span,
    children: Vec<SpanNode>,
    depth: usize,
}

impl SpanNode {
    fn from_trace(trace: &Trace) -> Vec<SpanNode> {
        let mut span_map: HashMap<SpanId, &Span> = HashMap::new();
        let mut children_map: HashMap<SpanId, Vec<SpanId>> = HashMap::new();
        let mut roots = Vec::new();

        // Build maps
        for span in &trace.spans {
            span_map.insert(span.span_id, span);
            if let Some(parent_id) = span.parent_span_id {
                children_map
                    .entry(parent_id)
                    .or_default()
                    .push(span.span_id);
            } else {
                roots.push(span.span_id);
            }
        }

        // Build tree recursively
        fn build_tree(
            span_id: SpanId,
            span_map: &HashMap<SpanId, &Span>,
            children_map: &HashMap<SpanId, Vec<SpanId>>,
            depth: usize,
        ) -> SpanNode {
            let span = span_map.get(&span_id).unwrap();
            let mut children = Vec::new();

            if let Some(child_ids) = children_map.get(&span_id) {
                // Sort children by start time
                let mut sorted_children = child_ids.clone();
                sorted_children
                    .sort_by_key(|id| span_map.get(id).map(|s| s.start_time.0).unwrap_or(0));

                for child_id in sorted_children {
                    children.push(build_tree(child_id, span_map, children_map, depth + 1));
                }
            }

            SpanNode {
                span: (*span).clone(),
                children,
                depth,
            }
        }

        roots
            .into_iter()
            .map(|root_id| build_tree(root_id, &span_map, &children_map, 0))
            .collect()
    }

    fn flatten(&self) -> Vec<(Span, usize, bool)> {
        let mut result = Vec::new();
        let has_children = !self.children.is_empty();
        result.push((self.span.clone(), self.depth, has_children));
        for child in &self.children {
            result.extend(child.flatten());
        }
        result
    }
}

/// Trace detail view - shows full trace information
#[component]
pub async fn TraceDetail(props: TraceDetailProps) -> View {
    let nav_state = props.nav_state;
    let trace_id = props.trace_id.clone();
    let session = props.session;

    // Fetch trace directly with async/await
    let client = HindsightServiceClient::new(session);
    let trace = client
        .get_trace(trace_id.clone()).await
        .expect("Failed to fetch trace")
        .expect("Trace not found");

    // Back button handler
    let on_back = move |_| {
        nav_state.navigate_to(Route::TraceList);
    };

    let title = trace
        .spans
        .iter()
        .find(|s| s.span_id == trace.root_span_id)
        .map(|s| s.name.clone())
        .unwrap_or_else(|| "Trace Detail".to_string());

    let nodes = SpanNode::from_trace(&trace);
    let flat_spans: Vec<_> = nodes.iter().flat_map(|n| n.flatten()).collect();

    view! {
        div(class="trace-detail") {
            div(class="detail-header") {
                button(class="btn", on:click=on_back) { "← Back" }
                div(class="detail-title") { (title) }
                div(class="detail-meta") {
                    span(class="trace-meta-label") { "id:" }
                    " "
                    span(style="font-family: var(--font-mono); font-size: var(--text-xs);") {
                        (trace_id.to_hex())
                    }
                }
            }

            div(class="detail-content") {
                div(class="waterfall") {
                    div(class="waterfall-header") {
                        div { "Operation" }
                        div { "Service" }
                        div { "Duration" }
                    }
                    (
                        flat_spans.clone().into_iter().map(|(span, depth, has_children)| {
                            span_row_view(span, depth, has_children)
                        }).collect::<Vec<_>>()
                    )
                }
            }
        }
    }
}

/// Create a span row view
fn span_row_view(span: Span, depth: usize, has_children: bool) -> View {
    let is_error = matches!(span.status, SpanStatus::Error { .. });

    let duration_text = if let Some(end) = span.end_time {
        let nanos = end.0.saturating_sub(span.start_time.0);
        let ms = nanos as f64 / 1_000_000.0;
        if ms < 1.0 {
            format!("{:.0}µs", nanos as f64 / 1_000.0)
        } else if ms < 10.0 {
            format!("{:.2}ms", ms)
        } else if ms < 1000.0 {
            format!("{:.1}ms", ms)
        } else {
            format!("{:.2}s", ms / 1000.0)
        }
    } else {
        "—".to_string()
    };

    view! {
        div(
            class="span-row",
            data-error=is_error.to_string(),
            data-has-children=has_children.to_string(),
            style=format!("--depth: {}", depth),
            tabindex="0"
        ) {
            div(class="span-name-container") {
                div(class="span-hierarchy-icon") { "▸" }
                div(class="span-name") { (span.name.clone()) }
            }
            div(class="span-service") { (span.service_name.clone()) }
            div(class="span-duration") { (duration_text) }
        }
    }
}

#[derive(Props)]
pub struct TraceDetailProps {
    pub trace_id: TraceId,
    pub nav_state: NavigationState,
    pub session: Arc<RpcSession>,
}

