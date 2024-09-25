use sycamore::prelude::*;

use super::atoms::notebook_icon::NotebookIcon;

#[component]
pub fn NoteCategories() -> View {
    view! {
        ul {
            li {
                div {
                    NotebookIcon()
                    "All Categories"
                }
            }
        }
    }
}
