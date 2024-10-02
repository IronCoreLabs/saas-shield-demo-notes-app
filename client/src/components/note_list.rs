use crate::{
    apis::{self, types::ListNotesResponse},
    components::{atoms::icon::Icon, search_notes::SEARCH_RESULT_CATEGORY},
    pages::index::{CurrentCategory, CurrentNote, SearchResults},
};
use sycamore::{futures::spawn_local_scoped, prelude::*, web::console_error};

#[component]
pub fn NoteList() -> View {
    let current_category = use_context::<Signal<CurrentCategory>>();
    let current_note = use_context::<Signal<CurrentNote>>();
    let search_results = use_context::<Signal<SearchResults>>();
    let notes_resp = create_signal(ListNotesResponse { result: vec![] });
    create_effect(move || {
        // if the current note changed we may have added one, check for server state
        current_note.track();
        let category = current_category.get_clone();
        // if the change here was the category to Search Results don't check for that category
        match category {
            CurrentCategory(Some(ref cn)) if cn == SEARCH_RESULT_CATEGORY => {
                // do nothing in the Search Results case
            }
            c => spawn_local_scoped(async move {
                match apis::notes(c.0).await {
                    Ok(resp) => notes_resp.set(resp),
                    Err(e) => {
                        console_error!("Failed to get notes: {:?}", e)
                    }
                };
            }),
        }
    });

    let notes = create_memo(move || notes_resp.get_clone().result);
    let current_category_name = create_memo(move || current_category.get_clone().0);
    let current_search_results = create_memo(move || search_results.get_clone().0);
    let notes_to_render = create_memo(move || {
        let mut render = notes.get_clone();
        match current_category_name.get_clone() {
            Some(cn) if &cn == SEARCH_RESULT_CATEGORY => {
                render = current_search_results.get_clone();
            }
            _ => {}
        }
        render
    });
    let notes_count = create_memo(move || notes_to_render.get_clone().len());

    view! {
        ul(class="w-full h-screen overflow-y-auto bg-gray-200") {
            li {
                div(class="flex flex-row flex-wrap items-center p-4 w-full h-24") {
                    h3(class="font-bold text-lg w-56") { (current_category.get_clone().0.unwrap_or_else(|| "All Notes".to_string())) }
                    div(on:click=move |_| current_category.set(CurrentCategory::default()), class="w-4") {
                        Icon(path="M6 18 18 6M6 6l12 12".to_string())
                    }
                    p(class="font-extralight text-gray-400 text-sm mt-auto mr-auto w-full") { (notes_count.get()) " notes"  }
                }
            }
            Indexed(
                list=notes_to_render,
                view=move |n| {
                    let set_note = move |_| current_note.set(CurrentNote(Some(n.id)));
                    let bg_color = create_memo(move || match current_note.get_clone().0 {
                        Some(cn) if cn == n.id => "bg-gray-300",
                        _ => ""
                    });
                    view! {
                        li {
                            div(
                                on:click=set_note,
                                class=format!("flex flex-col p-4 w-full h-24 border-t border-gray-400 {}", bg_color)
                            ) {
                                h4(class="font-bold text-sm") { (n.title.clone()) }
                                p(class="line-clamp-2 text-sm") { (n.body.clone()) }
                            }
                        }
                    }
                }
            )
        }
    }
}
