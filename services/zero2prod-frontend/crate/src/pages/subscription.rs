// Code using 
// * yew example 'password_strength'
// * register form https://tailwindcomponents.com/component/register-form-with-password-validator-tailwind-css-alpine-js
use yew::prelude::*;

pub enum Msg {
    SetPassword(String),
    RegeneratePassword,
}

#[derive(Debug, Default)]
pub struct Subscription {
    password: String,
}

impl Component for Subscription {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::default()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetPassword(next_password) => self.password = next_password,
            Msg::RegeneratePassword => self.password = "Secret".to_string()
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // let on_change = ctx.link().callback(Msg::SetPassword);
        // let onclick = ctx.link().callback(|_| Msg::RegeneratePassword);
        html! {
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
                                            <input placeholder="" type="text"
                                                   class="text-md block px-3 py-2 rounded-lg w-full
                                                   bg-white border-2 border-gray-300
                                                   placeholder-gray-600 shadow-md
                                                   focus:placeholder-gray-500 focus:bg-white
                                                   focus:border-gray-600 focus:outline-none"/>
                                        </div>
                                        <div class="py-1">
                                            <span class="px-1 text-sm text-gray-600">{"Email"}</span>
                                            <input placeholder="" type="email"
                                                   class="text-md block px-3 py-2 rounded-lg w-full
                                                   bg-white border-2 border-gray-300
                                                   placeholder-gray-600 shadow-md
                                                   focus:placeholder-gray-500 focus:bg-white
                                                   focus:border-gray-600 focus:outline-none"/>
                                        </div>
                                        <div class="flex justify-start">
                                            <label class="block text-gray-500 font-bold my-4 flex items-center">
                                                <input class="leading-loose text-pink-600 top-0" type="checkbox"/>
                                                <span class="ml-2 text-sm py-2 text-gray-600 text-left">{"Accept the"}
                                                      <a href="#"
                                                         class="font-semibold text-black border-b-2 border-gray-200
                                                         hover:border-gray-500">
                                                         {"Terms and Conditions of the site"}
                                                      </a>{"and"}
                                                      <a href="#"
                                                         class="font-semibold text-black border-b-2 border-gray-200
                                                         hover:border-gray-500">
                                                         {"the information data policy."}</a>
                                                </span>
                                            </label>
                                        </div>
                                        <button class="mt-3 text-lg font-semibold bg-gray-800 w-fulltext-white
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
        }
    }
}

