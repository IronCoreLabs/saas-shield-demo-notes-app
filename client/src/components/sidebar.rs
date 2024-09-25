use super::{note_categories::NoteCategories, org_avatar::OrgAvatar, search_notes::SearchNotes};
use sycamore::prelude::*;

#[component]
pub fn Sidebar() -> View {
    view! {
        div(class="grid grid-rows-[50px_50px_auto] bg-stone-700") {
            OrgAvatar(org_name="Org Name".to_string())
            SearchNotes()
            NoteCategories()
        }
    }
}
