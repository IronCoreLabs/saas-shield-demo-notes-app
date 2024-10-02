use crate::{
    apis,
    pages::index::{CurrentNote, CurrentOrg},
};
use std::fmt::Display;
use sycamore::{futures::spawn_local_scoped, prelude::*, web::console_error};
use web_sys::KeyboardEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sender {
    Bot,
    Human,
}

impl Display for Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Human => f.write_str("Human"),
            Self::Bot => f.write_str("Bot"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub sender: Sender,
    pub message: String,
    pub referenced_note_id: Option<usize>,
}

#[component]
pub fn Chatbot() -> View {
    let current_org = use_context::<Signal<CurrentOrg>>();
    let current_note = use_context::<Signal<CurrentNote>>();
    let visible_chatbot = create_signal(false);
    let loading = create_signal(false);
    let toggle_chatbot = move |_| {
        visible_chatbot.set(!visible_chatbot.get());
    };
    let current_query = create_signal(String::new());
    let chat_messages = create_signal(vec![ChatMessage {
        message: "I'm here to help you get more out of your notes, what would you like to know?"
            .to_string(),
        sender: Sender::Bot,
        referenced_note_id: None,
    }]);

    let submit_question = move || {
        let query = current_query.get_clone();
        chat_messages.update(|vs| {
            vs.push(ChatMessage {
                sender: Sender::Human,
                message: query,
                referenced_note_id: None,
            })
        });
        current_query.set(String::new());
        loading.set(true);
        spawn_local_scoped(async move {
            match apis::query_chatbot(chat_messages.get_clone()).await {
                Ok(response_message) => chat_messages.update(|vs| {
                    vs.push(ChatMessage {
                        sender: Sender::Bot,
                        message: response_message.response,
                        referenced_note_id: Some(response_message.note_id),
                    })
                }),
                Err(e) => {
                    console_error!("{e}");
                }
            };
            loading.set(false);
        });
    };

    // if the org changed clear the history
    create_effect(move || {
        current_org.track();
        chat_messages.set(vec![ChatMessage {
            message:
                "I'm here to help you get more out of your notes, what would you like to know?"
                    .to_string(),
            sender: Sender::Bot,
            referenced_note_id: None,
        }]);
        current_query.set(String::new());
        visible_chatbot.set(false);
    });

    view! {
        button(
            on:click=toggle_chatbot,
            r#type="button",
            class="absolute bottom-6 left-12 w-12 h-12 text-white bg-red-600 hover:bg-red-800 focus:ring-4 focus:outline-none focus:ring-red-300 font-medium rounded-full text-sm p-2.5 text-center inline-flex items-center me-2"
        ) {
            svg(class="w-7", aria-hidden="true", xmlns="http://www.w3.org/2000/svg", fill="none", viewBox="0 0 24 24") {
                path(stroke="currentColor", stroke-linecap="round", stroke-linejoin="round", stroke-width="2", d="M20.25 8.511c.884.284 1.5 1.128 1.5 2.097v4.286c0 1.136-.847 2.1-1.98 2.193-.34.027-.68.052-1.02.072v3.091l-3-3c-1.354 0-2.694-.055-4.02-.163a2.115 2.115 0 0 1-.825-.242m9.345-8.334a2.126 2.126 0 0 0-.476-.095 48.64 48.64 0 0 0-8.048 0c-1.131.094-1.976 1.057-1.976 2.192v4.286c0 .837.46 1.58 1.155 1.951m9.345-8.334V6.637c0-1.621-1.152-3.026-2.76-3.235A48.455 48.455 0 0 0 11.25 3c-2.115 0-4.198.137-6.24.402-1.608.209-2.76 1.614-2.76 3.235v6.226c0 1.621 1.152 3.026 2.76 3.235.577.075 1.157.14 1.74.194V21l4.155-4.155")
            }
        }
        (if visible_chatbot.get() {
            view! {
                div(class="absolute z-10 bottom-20 left-12 bg-white h-1/3 w-1/3 rounded flex flex-col text-sm border drop-shadow-lg") {
                    div(class="flex flex-row w-full grow overflow-y-auto pl-2 pr-2 pt-2 text-white") {
                        // TODO: scroll needs to follow the conversation
                        ul(class="flex flex-col w-full") {
                            Indexed(
                                list=chat_messages,
                                view=move |ChatMessage { sender, message, referenced_note_id }| {
                                    let direction = match sender {
                                        Sender::Human => "self-end bg-blue-500",
                                        Sender::Bot => "self-start bg-gray-500"
                                    };
                                    view! {
                                        li(class=format!("{direction} mb-1 rounded-lg p-2")) {
                                            (if let Some(note_id) = referenced_note_id {
                                                view! {
                                                    a(
                                                        on:click=move |_| current_note.set(CurrentNote(Some(note_id))),
                                                        class="underline"
                                                    ) {
                                                        "Based on this note."
                                                    }
                                                    " "
                                                }
                                            } else { view!() })

                                            (message)
                                        }
                                    }
                                }
                            )
                            (if loading.get() {
                                view! {
                                    li(class="self-start bg-gray-500 mb-1 rounded-lg p-2") {
                                        "..."
                                    }
                                }
                            } else { view!() })
                        }
                    }
                    div(class="flex flex-row w-full border-t h-8 p-2") {
                        input(
                            on:keypress=move |event: KeyboardEvent| {
                                if event.key().as_str() == "Enter" {
                                    submit_question();
                                }
                            },
                            bind:value=current_query,
                            r#type="text",
                            placeholder="Type a question...",
                            autocomplete="off",
                            class="outline-none p-2 rounded w-full")
                    }
                }
                // squirrel catcher if they click off the chat
                div(on:click=toggle_chatbot, class="h-screen w-screen absolute top-0 left-0")
            }
        } else { view!() })
    }
}
