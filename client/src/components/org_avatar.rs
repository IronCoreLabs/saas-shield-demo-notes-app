use super::atoms::office_icon::OfficeIcon;
use crate::pages::index::{CurrentCategory, CurrentNote, CurrentOrg};
use lazy_static::lazy_static;
use std::collections::HashMap;
use sycamore::prelude::*;
use wasm_bindgen::JsCast;

lazy_static! {
    pub static ref ORGS_MAP: HashMap<&'static str, &'static str> =
        HashMap::from([("notes-demo-2", "iguanatech"), ("notes-demo-1", "IronCore"),]);
}

#[component(inline_props)]
pub fn OrgAvatar() -> View {
    let current_org = use_context::<Signal<CurrentOrg>>();
    let current_note = use_context::<Signal<CurrentNote>>();
    let current_category = use_context::<Signal<CurrentCategory>>();
    // set the org cookie to whatever the org is, update it when the org changes
    create_effect(move || {
        let html_document: web_sys::HtmlDocument = window().document().unwrap().dyn_into().unwrap();
        html_document
            .set_cookie(&format!("organization={}", current_org.get_clone().0))
            .unwrap();
    });
    let current_org_name = create_memo(move || {
        *ORGS_MAP
            .get(current_org.get_clone().0.as_str())
            .expect("Un-hardcoded current org was set.")
    });
    let dropdown_visible = create_signal(false);
    let toggle_dropdown = move |_| {
        dropdown_visible.set(!dropdown_visible.get());
    };

    view! {
        button(on:click=toggle_dropdown,class="row-start-1 col-start-1 pl-2 p-1 mx-auto flex justify-start items-center space-x-2 text-white w-full") {
            div(class="relative w-7 h-7 rounded-full".to_string()) {
                OfficeIcon(class="absolute w-8 h-8 -left-1".to_string())
            }
            div(class="flex items-center") {
                p(class="text-sm") { (current_org_name.get_clone()) }
            }
        }
        (if dropdown_visible.get() {
            let dropdown_list_items = View::from(ORGS_MAP.iter().map(|(org_id, org_name)| view!(
                li {
                    div(on:click=move |e| {
                        current_org.set(CurrentOrg(org_id.to_string()));
                        current_note.set(CurrentNote::default());
                        current_category.set(CurrentCategory::default());
                        toggle_dropdown(e);
                    }, class="block px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-600 dark:hover:text-white") {
                        (*org_name)
                    }
                }
            )).collect::<Vec<_>>());

            view! {
                div(class="row-start-1 col-start-1 mt-12 h-24 z-10 bg-white divide-y divide-gray-100 rounded-lg shadow w-44 dark:bg-gray-700") {
                    ul(class="py-2 text-sm text-gray-700 dark:text-gray-200") {
                        (dropdown_list_items)
                    }
                }
                div(on:click=toggle_dropdown,class="h-screen w-screen absolute top-0 left-0")
            }
        } else {
            view! {}
        })
    }
}
