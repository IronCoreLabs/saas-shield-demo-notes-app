use crate::components::sidebar::Sidebar;
use sycamore::prelude::*;

#[component]
pub fn Index() -> View {
    view! {
        div(class="grid grid-cols-app h-screen w-screen") {
            Sidebar {}
            // NoteList {}
            div(class="bg-red-400") { "notelist" }
            // Note {}
            div(class="bg-orange-400") { "note" }
        }
    }
}
