use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    //log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
    let submit = ctx.link().callback(|_| Msg::SubmitMessage);

    html! {
        <div class="flex min-h-screen">
            // ── Sidebar Users ──
            <aside class="w-56 bg-gray-100 p-3 overflow-auto">
                <h2 class="text-xl mb-2">{"Users"}</h2>
                { for self.users.iter().map(|u| html! {
                    <div class="flex items-center bg-white rounded-lg p-2 mb-3">
                        <img
                            class="w-12 h-12 rounded-full"
                            src={u.avatar.clone()}
                            alt="avatar"
                        />
                        <div class="ml-3">
                            <div class="font-semibold">{ &u.name }</div>
                            <div class="text-xs text-gray-400">{"Hi there!"}</div>
                        </div>
                    </div>
                }) }
            </aside>

            // ── Chat Area ──
            <main class="flex-grow flex flex-col">
                <header class="h-14 flex items-center px-4 border-b">
                    <h2 class="text-xl">{"💬 Chat!"}</h2>
                </header>

                <section class="flex-grow overflow-auto px-4 py-2 border-b">
                    { for self.messages.iter().map(|m| {
                        let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                        html! {
                            <div class="flex items-start mb-4">
                                <img
                                    class="w-8 h-8 rounded-full mr-2"
                                    src={user.avatar.clone()}
                                    alt="avatar"
                                />
                                <div>
                                    <div class="text-sm font-medium">{ &m.from }</div>
                                    <div class="text-xs text-gray-500">
                                        { if m.message.ends_with(".gif") {
                                            html! {
                                                <img
                                                    class="mt-2 w-32 h-32 rounded"
                                                    src={m.message.clone()}
                                                    alt="gif"
                                                />
                                            }
                                        } else {
                                            html! { &m.message }
                                        }}
                                    </div>
                                </div>
                            </div>
                        }
                    }) }
                </section>

                <footer class="h-14 flex items-center px-4">
                    <input
                        ref={self.chat_input.clone()}
                        type="text"
                        placeholder="Message"
                        class="flex-grow py-2 px-4 mr-2 bg-gray-100 rounded-full outline-none"
                    />
                    <button
                        onclick={submit}
                        class="w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white shadow"
                    >
                        <svg viewBox="0 0 24 24" class="w-6 h-6 fill-current">
                            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
                        </svg>
                    </button>
                </footer>
            </main>
        </div>
    }
}
}