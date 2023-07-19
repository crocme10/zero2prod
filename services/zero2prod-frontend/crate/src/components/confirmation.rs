use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};
use yew::{html, Component, Context, Html};
use yew_router::prelude::*;

use crate::app::Route;
use crate::components::{FetchError, FetchState};

// HTML / CSS after https://tailwindcomponents.com/component/register-form-with-password-validator-tailwind-css-alpine-js

const CONFIRMATION_URL: &str = "http://localhost:8081/api/subscriptions/confirmation";

async fn submit_subscription_confirmation(url: &str) -> Result<String, FetchError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    //opts.mode(RequestMode::Cors); // FIXME Why Cors ?

    let request = Request::new_with_str_and_init(url, &opts).map_err(|_| FetchError {
        description: "Could not build a request".to_string(),
    })?;
    request
        .headers()
        .set("Accept-Encoding", "gzip, deflate, br")
        .map_err(|_| FetchError {
            description: "Could not set header".to_string(),
        })?;
    request
        .headers()
        .set("Accept", "application/json")
        .map_err(|_| FetchError {
            description: "Could not set header".to_string(),
        })?;
    gloo_console::log!("Submitting confirmation request");
    let window = gloo::utils::window();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| FetchError {
            description: "Could not fetch response".to_string(),
        })?;

    let resp: Response = resp_value.dyn_into().map_err(|_| FetchError {
        description: "Could not cast response".to_string(),
    })?;

    gloo_console::log!("resp");
    let value = JsFuture::from(resp.json().map_err(|_| FetchError {
        description: "Could not extract json from response".to_string(),
    })?)
    .await
    .map_err(|_| FetchError {
        description: "Could not turn Json to JsValue".to_string(),
    })?;
    gloo_console::log!("value");

    let res: SubResp = serde_wasm_bindgen::from_value(value).map_err(|_| FetchError {
        description: "Could not deserialize response".to_string(),
    })?;
    gloo_console::log!("serde_wasm_bindgen");

    match res {
        SubResp::Success(sub) => Ok(sub.status),
        SubResp::Fail(err) => Err(err),
    }
}

pub enum Msg {
    SetConfirmationState(FetchState<String>),
}

pub struct Confirmation {
    state: FetchState<String>,
}

impl Component for Confirmation {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        gloo_console::log!("Creating");
        let token = ctx.link().location().unwrap().query_str().to_string();
        let url = format!("{}{}", CONFIRMATION_URL, token);
        // FIXME We need to massage that path, so that it targets the backend.
        gloo_console::log!("Path: ", url.clone());
        ctx.link().send_future(async move {
            let url = url.clone();
            match submit_subscription_confirmation(url.as_str()).await {
                Ok(resp) => Msg::SetConfirmationState(FetchState::Success(resp)),
                Err(err) => Msg::SetConfirmationState(FetchState::Failed(err)),
            }
        });
        Self {
            state: FetchState::Fetching,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetConfirmationState(new_state) => {
                self.state = new_state;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        match &self.state {
            FetchState::NotFetching => html! {
                <div class="w-full relative">
                    <div class="md:mt-6">
                        <div class="text-center font-semibold text-black">
                            {"Your confirmation is about to be processed..."}
                        </div>
                    </div>
                </div>
            },
            FetchState::Fetching => html! {
            <div class="w-full relative">
                <div class="md:mt-6">
                    <div class="text-center font-semibold text-black">
                        {"Your confirmation is processed..."}
                    </div>
                </div>
            </div>
            },
            FetchState::Success(_data) => html! {
            <div class="w-full relative">
                <div class="md:mt-6">
                    <div class="text-center font-semibold text-black text-lg">
                        {"Your subscription is"}<br/>{"confirmed"}
                    </div>
                    <div class="text-center font-base text-black mt-8">
                        {"You will start receiving newsletters at email ADDRESS"}
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
                    <div class="text-center font-semibold text-black">
                        {"An error occured while processing your confirmation"}
                    </div>
                    <div class="text-center font-base text-black">
                    {err.description.clone()}
                    </div>
                    <div class="text-center font-base">
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
    Success(SubscriptionConfirmationResp),
}

/// This is what we return to the user in response to the subscription request.
/// Currently this is just a placeholder, and it does not return any useful
/// information.
/// FIXME Share code with frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionConfirmationResp {
    pub status: String,
}
