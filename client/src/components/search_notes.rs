use super::atoms::search_icon::SearchIcon;
use crate::{
    apis,
    pages::index::{CurrentCategory, SearchResults},
};
use sycamore::{futures::spawn_local_scoped, prelude::*, web::console_error};
use web_sys::KeyboardEvent;

pub const SEARCH_RESULT_CATEGORY: &str = "Search Results";

#[component]
pub fn SearchNotes() -> View {
    let search_results = use_context::<Signal<SearchResults>>();
    let current_category = use_context::<Signal<CurrentCategory>>();
    let search = create_signal(String::new());
    let execute_search = move || {
        let query = search.get_clone();
        spawn_local_scoped(async move {
            if query != String::new() {
                match apis::search(query).await {
                    Ok(resp) => {
                        search_results.set(SearchResults(resp.result));
                        // BIG HACK ALERT
                        current_category
                            .set(CurrentCategory(Some(SEARCH_RESULT_CATEGORY.to_string())));
                    }
                    Err(e) => {
                        console_error!("{:?}", e)
                    }
                };
            }
        })
    };

    view! {
        div(class="w-4/5 max-w-64 flex justify-end items-center relative h-14 text-white") {
            input(on:keypress=move |event: KeyboardEvent| {
                if event.key().as_str() == "Enter" {
                    execute_search()
                }
            }, bind:value=search,placeholder="Search notes...", class="bg-stone-600 rounded-sm w-full p-2 text-sm", bind:value=search)
            div(on:click=move |_| execute_search(),class="absolute mr-2 w-4") {
                SearchIcon()
            }
        }
    }
}
