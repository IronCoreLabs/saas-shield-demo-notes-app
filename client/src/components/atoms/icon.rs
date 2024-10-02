use sycamore::prelude::*;

#[component(inline_props)]
pub fn Icon(color: Option<String>, class: Option<String>, path: String) -> View {
    let concrete_color = color.unwrap_or_else(|| "currentcolor".to_string());
    let concrete_class = class.unwrap_or_else(|| "".to_string());

    view! {
        svg(xmlns="http://www.w3.org/2000/svg", fill="none", viewBox="0 0 24 24", stroke-width="1.5", stroke="currentColor", class=format!("{concrete_class}")) {
            g(color=concrete_color) {
                path(stroke-linecap="round", stroke-linejoin="round", d=path)
            }
        }
    }
}
