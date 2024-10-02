use crate::{
    apis::types::Note,
    components::{chatbot::Chatbot, note::Note, note_list::NoteList, sidebar::Sidebar},
};
use sycamore::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct CurrentOrg(pub String);
#[derive(Clone, PartialEq, Eq, Default)]
pub struct CurrentCategory(pub Option<String>);
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct CurrentNote(pub Option<usize>);
#[derive(Clone, PartialEq, Eq, Default)]
pub struct SearchResults(pub Vec<Note>);

#[component]
pub fn Index() -> View {
    // App State
    let current_org = create_signal(CurrentOrg("notes-demo-1".to_string()));
    let current_category = create_signal(CurrentCategory::default());
    let current_note = create_signal(CurrentNote::default());
    let search_results = create_signal(SearchResults::default());
    provide_context(current_org);
    provide_context(current_category);
    provide_context(current_note);
    provide_context(search_results);

    view! {
        div(class="grid grid-cols-app h-screen w-screen") {
            Sidebar()
            NoteList()
            Note()
            Chatbot()
        }
    }
}
