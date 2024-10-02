use crate::{
    apis::{self, types::ListCategoriesResponse},
    components::atoms::{note_icon::NoteIcon, notebook_icon::NotebookIcon},
    pages::index::CurrentCategory,
};
use sycamore::{futures::spawn_local_scoped, prelude::*, web::console_error};

const SELECTED: &'static str = "bg-stone-600";

#[component]
pub async fn NoteCategories() -> View {
    let current_category = use_context::<Signal<CurrentCategory>>();
    let categories_resp = create_signal(ListCategoriesResponse { result: vec![] });
    create_effect(move || {
        current_category.track();
        spawn_local_scoped(async move {
            match apis::categories().await {
                Ok(resp) => categories_resp.set(resp),
                Err(e) => {
                    console_error!("{:?}", e)
                }
            };
        })
    });
    let categories = create_memo(move || categories_resp.get_clone().result);
    let all_category_bg = create_memo(move || {
        if current_category.get_clone().0.is_none() {
            SELECTED
        } else {
            ""
        }
    });

    view! {
        ul(class="text-white w-full") {
            li {
                button(
                    on:click=move |_| current_category.set(CurrentCategory(None)),
                    class=format!("pl-2 w-full max-w-full flex items-center h-14 {}", all_category_bg)
                ) {
                    NotebookIcon()
                    "All Notes"
                }
            }
            Indexed(
                list=categories,
                view=move |c| {
                    let button_category = c.clone();
                    let iter_category = c.clone();
                    let selected_category_class = create_memo(move || match current_category.get_clone().0 {
                        Some(selected_category) if selected_category == iter_category => SELECTED,
                        _ => ""
                    });
                    view! {
                        li {
                            button(
                                on:click=move |_| current_category.set(CurrentCategory(Some(button_category.clone()))),
                                class=format!("pl-2 w-full max-w-full flex items-center h-14 {}", selected_category_class)
                            ) {
                                NoteIcon()
                                (c.clone())
                            }
                        }
                    }
                },
            )
        }
    }
}
