use crate::pages::index::CurrentNote;

use super::atoms::add_icon::AddIcon;
use sycamore::prelude::*;

#[component]
pub fn AddNote() -> View {
    let current_note = use_context::<Signal<CurrentNote>>();
    let handle_click = move |_| {
        current_note.set(CurrentNote(None));
    };

    view! {
        button(
            class="pl-4 w-full max-w-full flex items-center h-14",
            on:click=handle_click
        ) {
            AddIcon(class="mr-2 w-8".to_string())
            p(class="text-white text-md") { "New Note" }
        }
    }
}
