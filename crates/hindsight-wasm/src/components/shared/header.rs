//! Application header component

use sycamore::prelude::*;

/// Main application header
#[component]
pub fn Header(props: HeaderProps) -> View {
    let connection_status = props.connection_status;

    view! {
        header(class="header") {
            h1 {
                img(src="/static/logo.svg", alt="Hindsight logo")
                " hindsight"
            }
            div(class="status-badge") {
                div(class="status-dot") {}
                span { (connection_status.get_clone()) }
            }
        }
    }
}

#[derive(Props)]
pub struct HeaderProps {
    pub connection_status: Signal<String>,
}
