use sycamore::prelude::*;

use super::{
    add_note::AddNote, note_categories::NoteCategories, org_avatar::OrgAvatar,
    search_notes::SearchNotes,
};

#[component]
pub fn Sidebar() -> View {
    view! {
        div(class="grid grid-rows-[50px_50px_50px_auto] justify-items-center bg-stone-900 h-screen overflow-y-auto") {
            OrgAvatar()
            SearchNotes()
            AddNote()
            NoteCategories()
        }
    }
}
