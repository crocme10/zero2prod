use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, Request, RequestInit, RequestMode, Response};
use yew::{html, Component, Context, Html, NodeRef};
use zero2prod_common::subscriptions::{SubscriptionRequest, SubscriptionsResp};

use crate::components::{FetchError, FetchState};

const SUBSCRIPTION_URL: &str = "http://localhost:8081/api/subscriptions";

async fn submit_subscription(
    url: &'static str,
    request: SubscriptionRequest,
) -> Result<SubscriptionsResp, FetchError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    let value = serde_json::to_string(&request).unwrap();
    // let value = serde_wasm_bindgen::to_value(&request)?;
    opts.body(Some(&JsValue::from_str(&value)));
    //opts.mode(RequestMode::Cors); // FIXME Why Cors ?

    let request = Request::new_with_str_and_init(url, &opts)?;
    request
        .headers()
        .set("Accept-Encoding", "gzip, deflate, br")?;
    request.headers()
        .set("Content-Type", "application/json")?;
    request.headers()
        .set("Accept", "application/json")?;
    gloo_console::log!("Submitting request");
    let window = gloo::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    let value = JsFuture::from(resp.json()?).await?;

    let resp = serde_wasm_bindgen::from_value(value)?;
    Ok(resp)
}

pub enum Msg {
    SetSubscriptionState(FetchState<SubscriptionsResp>),
    Submit,
    HoverIndex(usize),
}

pub struct Subscription {
    state: FetchState<SubscriptionsResp>,
    refs: Vec<NodeRef>,
    focus_index: usize,
}

impl Subscription {
    fn apply_focus(&self) {
        gloo_console::log!("Applying focus to {}", JsValue::from(self.focus_index));
        if let Some(input) = self.refs[self.focus_index].cast::<HtmlInputElement>() {
            input.focus().unwrap();
        }
    }
}

impl Component for Subscription {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            state: FetchState::NotFetching,
            refs: vec![NodeRef::default(), NodeRef::default()],
            focus_index: 0,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.apply_focus();
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HoverIndex(index) => {
                gloo_console::log!("Setting focus index to {}", JsValue::from(index));
                self.focus_index = index;
                self.apply_focus();
                false
            }

            Msg::SetSubscriptionState(new_state) => {
                self.state = new_state;
                true
            }
            Msg::Submit => {
                let username = &self.refs[0];
                let email = &self.refs[1];
                let username_value = username.cast::<HtmlInputElement>().unwrap().value();
                let email_value = email.cast::<HtmlInputElement>().unwrap().value();
                gloo_console::log!(
                    "Retrieved form data",
                    JsValue::from(username_value.clone()),
                    JsValue::from(email_value.clone())
                );
                ctx.link().send_future(async {
                    let request = SubscriptionRequest {
                        username: username_value,
                        email: email_value,
                    };
                    // TODO Validate subscription
                    match submit_subscription(SUBSCRIPTION_URL, request).await {
                        Ok(resp) => Msg::SetSubscriptionState(FetchState::Success(resp)),
                        Err(err) => Msg::SetSubscriptionState(FetchState::Failed(err)),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetSubscriptionState(FetchState::Fetching));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.state {
            FetchState::NotFetching => html! {
                <div class="container max-w-full mx-auto md:py-24 px-6">
                    <div class="max-w-sm mx-auto px-6">
                        <div class="relative flex flex-wrap">
                            <div class="w-full relative">
                                <div class="md:mt-6">
                                    <div class="text-center font-semibold text-black">
                                        {"Register to Zero2Prod Newsletter"}
                                    </div>
                                    <div class="text-center font-base text-black">
                                        {"Fill in the following details to register"}
                                    </div>
                                    <form class="mt-8">
                                        <div class="mx-auto max-w-lg ">
                                            <div class="py-1">
                                                <span class="px-1 text-sm text-gray-600">{"Username"}</span>
                                                <input
                                                  ref={&self.refs[0]}
                                                  placeholder="username"
                                                  type="text"
                                                  onmouseover={ctx.link().callback(|_| Msg::HoverIndex(0))}
                                                  class="text-md block px-3 py-2 rounded-lg w-full
                                                   bg-white border-2 border-gray-300
                                                   placeholder-gray-600 shadow-md
                                                   focus:placeholder-gray-500 focus:bg-white
                                                   focus:border-gray-600 focus:outline-none"/>
                                            </div>
                                            <div class="py-1">
                                                <span class="px-1 text-sm text-gray-600">{"Email"}</span>
                                                <input
                                                  ref={&self.refs[1]}
                                                  placeholder="username@domain.com"
                                                  type="email"
                                                  onmouseover={ctx.link().callback(|_| Msg::HoverIndex(1))}
                                                  class="text-md block px-3 py-2 rounded-lg w-full
                                                   bg-white border-2 border-gray-300
                                                   placeholder-gray-600 shadow-md
                                                   focus:placeholder-gray-500 focus:bg-white
                                                   focus:border-gray-600 focus:outline-none"/>
                                            </div>
                                            <div class="flex justify-start">
                                                <label class="block text-gray-500 font-bold my-4 flex items-center">
                                                    <input class="leading-loose text-pink-600 top-0" type="checkbox"/>
                                                    <span class="ml-2 text-sm py-2 text-gray-600 text-left">{"Accept the "}
                                                          <a href="#"
                                                             class="font-semibold text-black border-b-2 border-gray-200
                                                             hover:border-gray-500">
                                                             {"Terms and Conditions of the site"}
                                                          </a>{" and "}
                                                          <a href="#"
                                                             class="font-semibold text-black border-b-2 border-gray-200
                                                             hover:border-gray-500">
                                                             {"the information data policy."}</a>
                                                    </span>
                                                </label>
                                            </div>
                                            <button
                                              onclick={ctx.link().callback(|_| Msg::Submit)}
                                              class="mt-3 text-lg font-semibold bg-gray-800 w-fulltext-white
                                            rounded-lg px-6 py-3 block shadow-xl hover:text-white hover:bg-black">
                                               {"Register"}
                                            </button>
                                        </div>
                                    </form>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            },
            FetchState::Fetching => html! { "Processing subscription" },
            FetchState::Success(_data) => html! { "Success" },
            FetchState::Failed(err) => html! { err },
        }
    }
}
