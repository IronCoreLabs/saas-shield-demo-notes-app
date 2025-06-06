use sycamore::prelude::*;

use super::icon::Icon;

#[component(inline_props)]
pub fn OfficeIcon(class: Option<String>) -> View {
    view! {
        Icon(
            class=class.unwrap_or_else(|| "".to_string()),
            path="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21".to_string()
        )
    }
}
