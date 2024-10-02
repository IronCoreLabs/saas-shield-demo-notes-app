use crate::{
    apis::{
        self, create_attachment, create_note, rekey_note, types::AttachmentInfo, update_note,
        write_to_url,
    },
    components::atoms::{attachment_icon::AttachmentIcon, download_icon::DownloadIcon},
    pages::index::{CurrentCategory, CurrentNote},
};
use sycamore::{futures::spawn_local_scoped, prelude::*};
use web_sys::window;

#[component]
pub fn Note() -> View {
    let current_note_id = use_context::<Signal<CurrentNote>>();
    let current_category = use_context::<Signal<CurrentCategory>>();
    let category = create_signal(String::new());
    let title = create_signal(String::new());
    let body = create_signal(String::new());
    let id = create_signal("Note ID".to_string());
    let attachments = create_signal(Vec::<AttachmentInfo>::new());
    create_effect(move || {
        let current_note = current_note_id.get();
        spawn_local_scoped(async move {
            if let Some(note_id) = current_note.0 {
                match apis::note(note_id).await {
                    Ok(resp) => {
                        if let Some(note) = resp {
                            category.set(note.category.unwrap_or_else(|| String::new()));
                            title.set(note.title);
                            body.set(note.body);
                            id.set(format!("Note ID {}", note.id.to_string()));
                            attachments.set(note.attachments);
                        }
                    }
                    Err(_) => {}
                };
            } else {
                category.set(String::new());
                title.set(String::new());
                body.set(String::new());
                id.set("Note ID".to_string());
                attachments.set(vec![])
            }
        })
    });

    let save_note = move |_| {
        spawn_local_scoped(async move {
            let category_str = category.get_clone();
            let category_to_send = (!category_str.is_empty()).then(|| category_str);
            let attachment_ids = attachments.get_clone().iter().map(|a| a.id).collect();
            let operation = if let Some(note_to_update) = current_note_id.get().0 {
                update_note(
                    note_to_update,
                    title.get_clone(),
                    body.get_clone(),
                    category_to_send,
                    attachment_ids,
                )
                .await
            } else {
                create_note(
                    title.get_clone(),
                    body.get_clone(),
                    category_to_send,
                    attachment_ids,
                )
                .await
            };

            match operation {
                Ok(note) => {
                    current_note_id.set(CurrentNote(Some(note.id)));
                    // touch the category, ideally this would only be if it was one that didn't already exist
                    current_category.set(current_category.get_clone());
                    attachments.set(note.attachments);
                }
                Err(_) => {}
            }
        })
    };

    let rekey_note_handler = move |_| {
        spawn_local_scoped(async move {
            if let Some(note_to_update) = current_note_id.get().0 {
                rekey_note(note_to_update).await.unwrap();
            };
        })
    };

    let new_attachment = move |_| {
        spawn_local_scoped(async move {
            use rfd::AsyncFileDialog;
            let (file_bytes, filename) = async {
                let maybe_file = AsyncFileDialog::new().set_directory("/").pick_file().await;
                let file = maybe_file.unwrap();
                let data = file.read().await;
                let filename = file.file_name().to_string();
                (data, filename)
            }
            .await;

            let maybe_attachment_response = create_attachment(filename).await;
            match maybe_attachment_response {
                Ok(attachment_response) => {
                    write_to_url(attachment_response.presigned_put_url, file_bytes)
                        .await
                        .unwrap();
                    attachments.update(|v| {
                        v.push(AttachmentInfo {
                            filename: attachment_response.filename,
                            id: attachment_response.id,
                            url: attachment_response.url,
                        })
                    });
                }
                Err(_) => {}
            }
            {}
        })
    };

    view! {
        div(class="border-l drop-shadow-md flex flex-col") {
            input(bind:value=title, r#type="text", placeholder="Title", autocomplete="off", class="w-full h-12 pl-4 outline-none border-b text-lg")
            textarea(bind:value=body, placeholder="Note", class="w-full grow pl-4 pt-2 outline-none text-sm")
            div(class="bg-white") {
                button(on:click=new_attachment, class="ml-1"){ AttachmentIcon(class="w-6".to_string()) }
                // TODO: delete button, not totally mvp necessary
                (attachments.get_clone().into_iter().map(move |a| view!{button(class="ml-1 w-6", on:click= move |_| {window().unwrap().open_with_url(a.url.clone().as_str()).unwrap();} ){ DownloadIcon()  } } ).collect::<Vec<_>>())
            }
            div(class="flex flex-row bg-white") {
                input(bind:value=category, r#type="text", placeholder="Category", class="w-full h-12 pl-4 outline-none text-sm border-t")
                div(class="flex text-nowrap text-sm border-t border-l text-gray-400 pl-4 pr-4 items-center") {
                    (id.get_clone())
                }
            }
            div(class="bg-white") {
                (if current_note_id.get().0.is_none() {
                  view!{button(on:click=save_note, class="w-full border-2 bg-red-600 text-white h-12 self-end"){ "Save" }}
                } else {
                    view!{
                        button(on:click=save_note, class="w-5/6 border-2 bg-red-600 text-white h-12 self-end"){ "Save" }
                        button(on:click=rekey_note_handler, class="w-1/6 border-2 bg-red-600 text-white h-12 self-end"){ "Rekey" }
                    }
                })
            }
        }
    }
}
