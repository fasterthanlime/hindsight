//! Tab bar component for top-level navigation

use sycamore::prelude::*;

use crate::navigation::{NavigationState, TabId};

/// Tab bar component - renders the top-level navigation tabs
#[component]
pub fn TabBar(props: TabBarProps) -> View {
    let nav_state = props.nav_state;

    view! {
        div(class="tab-bar") {
            Indexed(
                list=nav_state.available_tabs,
                view=move |tab| {
                    let nav_state = nav_state.clone();
                    view! {
                        Tab(tab=tab, nav_state=nav_state)
                    }
                }
            )
        }
    }
}

#[derive(Props)]
pub struct TabBarProps {
    pub nav_state: NavigationState,
}

/// Individual tab component
#[component]
fn Tab(TabProps { tab, nav_state }: TabProps) -> View {
    // Check if this tab is active
    let is_active = create_memo(move || nav_state.active_tab.with(|active| *active == tab));

    // Click handler
    let on_click = move |_| {
        nav_state.navigate_to_tab(tab);
    };

    let class = create_memo(move || {
        if is_active.with(|active| *active) {
            "tab active"
        } else {
            "tab"
        }
    });

    view! {
        button(
            class=class.with(|c| *c),
            on:click=on_click,
        ) {
            (tab.label())
        }
    }
}

#[derive(Props)]
struct TabProps {
    tab: TabId,
    nav_state: NavigationState,
}
