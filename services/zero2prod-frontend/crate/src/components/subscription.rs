use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, Request, RequestInit, Response};
use yew::{html, Component, Context, Html, NodeRef};
use yew_router::prelude::*;
use zero2prod_common::subscriptions::{SubscriptionRequest, SubscriptionsResp};

use crate::app::Route;
use crate::components::{FetchError, FetchState};

// HTML / CSS after https://tailwindcomponents.com/component/register-form-with-password-validator-tailwind-css-alpine-js

const Z2P_BACKEND_URL: &str = "http://localhost:8081";

/// This function takes the subscription request obtained from the form fields,
/// and submits the request to the backend. It then casts the JSON response
/// from the backend into a SubscriptionResponse, or an error, which can be
/// displayed to the user.
async fn submit_subscription(
    request: SubscriptionRequest,
) -> Result<SubscriptionsResp, FetchError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    //opts.mode(RequestMode::Cors); // FIXME Why Cors ?

    // FIXME Serializing and Deserializing so that the backend
    // accepts this as JSON. I should just pass the value??
    // FIXME Too much error code, should maybe use a context.
    let value = serde_json::to_string(&request).unwrap();
    opts.body(Some(&JsValue::from_str(&value)));

    let url = format!("{}/api/subscriptions", Z2P_BACKEND_URL);

    let request = Request::new_with_str_and_init(&url, &opts).map_err(|_| FetchError {
        status_code: 400,
        description: "Could not build a request".to_string(),
    })?;
    request
        .headers()
        .set("Accept-Encoding", "gzip, deflate, br")
        .map_err(|_| FetchError {
            status_code: 400,
            description: "Could not set header".to_string(),
        })?;
    request
        .headers()
        .set("Content-Type", "application/json")
        .map_err(|_| FetchError {
            status_code: 400,
            description: "Could not set header".to_string(),
        })?;
    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|_| FetchError {
            status_code: 400,
            description: "Could not set header".to_string(),
        })?;
    gloo_console::log!("Submitting request");
    let window = gloo::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| FetchError {
            status_code: 400,
            description: "Could not fetch response".to_string(),
        })?;

    let resp: Response = resp_value.dyn_into().map_err(|_| FetchError {
        status_code: 400,
        description: "Could not cast response".to_string(),
    })?;

    gloo_console::log!("resp");
    let value = JsFuture::from(resp.json().map_err(|_| FetchError {
        status_code: 400,
        description: "Could not extract json from response".to_string(),
    })?)
    .await
    .map_err(|_| FetchError {
        status_code: 400,
        description: "Could not turn Json to JsValue".to_string(),
    })?;
    gloo_console::log!("value");

    let res: SubResp = serde_wasm_bindgen::from_value(value).map_err(|_| FetchError {
        status_code: 400,
        description: "Could not deserialize response".to_string(),
    })?;
    gloo_console::log!("serde_wasm_bindgen");

    match res {
        SubResp::Success(sub) => Ok(sub),
        SubResp::Fail(err) => Err(err),
    }
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
                    match submit_subscription(request).await {
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
                <div class="w-full relative">
                    <div class="md:mt-6">
                        <div class="text-center font-text font-semibold text-black text-lg">
                            {"Register to Zero2Prod Newsletter"}
                        </div>
                        <div class="mt-4 text-center font-text text-black">
                            {"Fill in the following details to register"}
                        </div>
                        <form class="mt-8">
                            <div class="mx-auto max-w-lg ">
                                <div class="py-1">
                                    <span class="px-1 text-sm font-text text-gray-600">{"Username"}</span>
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
                                    <span class="px-1 text-sm font-text text-gray-600">{"Email"}</span>
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
                                        <span class="ml-2 text-sm py-2 text-gray-600 text-left font-text">{"Accept the "}
                                              <Link<Route> to={Route::TermsAndConditions}
                                                 classes="font-semibold text-black border-b-2 border-gray-200
                                                 hover:border-gray-500">
                                                 {"Terms and Conditions of the site"}
                                              </Link<Route>>{" and "}
                                              <a href="#"
                                                 class="font-semibold text-black border-b-2 border-gray-200
                                                 hover:border-gray-500">
                                                 {"the information data policy."}</a>
                                        </span>
                                    </label>
                                </div>
                                <div class="flex justify-between">
                                    <button
                                      onclick={ctx.link().callback(|_| Msg::Submit)}
                                      class="mt-3 text-lg font-semibold bg-gray-800 text-white
                                    rounded-lg px-6 py-3 block shadow-xl hover:text-white hover:bg-black">
                                       {"Register"}
                                    </button>
                                    <Link<Route> to={Route::Home}
                                      classes="mt-3 text-lg font-semibold bg-gray-800 text-white
                                      rounded-lg px-6 py-3 inline-block shadow-xl hover:text-white hover:bg-black">
                                       {"Cancel"}
                                    </Link<Route>>
                                </div>
                            </div>
                        </form>
                    </div>
                </div>
            },
            FetchState::Fetching => html! {
            <div class="w-full relative">
                <div class="md:mt-6">
                    <div class="text-center font-semibold text-black">
                        {"Your subscription is processed..."}
                    </div>
                </div>
            </div>
            },
            FetchState::Success(_data) => html! {
            <div class="w-full relative">
                <div class="md:mt-6">
                    <div class="text-center font-text font-semibold text-black text-lg">
                        {"Your subscription is"}<br/>{"pending confirmation"}
                    </div>
                    <div class="text-center font-text text-black mt-8">
                        {"You will shortly receive an email ADDRESS with a link you need to follow in order to finalize your subscription"}
                    </div>
                    <div class="text-center font-base">
                        <Link<Route> to={Route::Home}
                          classes="mt-8 text-lg font-semibold bg-gray-800 text-white
                          rounded-lg px-6 py-3 inline-block shadow-xl hover:text-white hover:bg-black">
                           {"Home"}
                        </Link<Route>>
                    </div>
                </div>
            </div>
            },
            FetchState::Failed(err) => html! {
            <div class="w-full relative">
                <div class="md:mt-6">
                    <div class="text-center font-text font-semibold text-black text-lg">
                        {"An error occured while"}<br/>{"processing your subscription"}
                    </div>
                    <div class="text-center font-text text-black mt-8">
                        {err.description.clone()}
                    </div>
                    <div class="mt-8 text-center font-base">
                        <Link<Route> to={Route::Home}
                          classes="mt-3 text-lg font-semibold bg-gray-800 text-white
                          rounded-lg px-6 py-3 inline-block shadow-xl hover:text-white hover:bg-black">
                           {"Home"}
                        </Link<Route>>
                    </div>
                </div>
            </div>
            },
        }
    }
}

// This is an enum to properly catch a backend
// response.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum SubResp {
    Fail(FetchError),
    Success(SubscriptionsResp),
}
