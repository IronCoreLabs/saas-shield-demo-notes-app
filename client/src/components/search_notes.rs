use super::atoms::search_icon::SearchIcon;
use sycamore::prelude::*;

#[component]
pub fn SearchNotes() -> View {
    let search = create_signal(String::new());

    view! {
        div(class="w-2/3 flex justify-end items-center relative") {
            input(placeholder="Search notes...", class="border border-gray-400 rounded-md p-4 w-full", bind:value=search)
            SearchIcon(class="absolute mr-2 w-10".to_string()) {}
        }
    }
}
