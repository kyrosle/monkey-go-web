use std::{
    error::Error,
    fmt::{Debug, Display},
    time::Duration,
};

use gloo_console::log;
use gloo_net::http::Request;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, RequestInit, Response};
use yew::prelude::*;

const URL: &str = "/api/code";

#[derive(Debug, Clone, PartialEq)]
pub struct SendError {
    err: String,
}

impl SendError {
    pub fn new<T: Debug>(err: T) -> Self {
        SendError {
            err: format!("{err:?}"),
        }
    }
}

impl Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.err, f)
    }
}
impl Error for SendError {}

pub struct App {
    code: String,
    message: Option<String>,
    error: Option<String>,
    show_message: bool,
    show_error: bool,
}

pub enum AppMsg {
    Response(String),
    ResponseError(SendError),
    RunCode,
    Fetching,
    CodeChanged(String),
    TurnOffShow,
}

async fn send_code(url: &'static str, value: String) -> Result<String, SendError> {
    let text = Request::post(url)
        .body(value)
        .send()
        .await
        .map_err(SendError::new)?;
    let text = text.text().await.map_err(SendError::new)?;
    Ok(text)
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App {
            code: PLACEHOLDER.to_string(),
            message: Some("Info".to_string()),
            error: Some("Error".to_string()),
            show_error: false,
            show_message: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Response(ret) => {
                self.message = Some(ret);
                self.show_message = true;
                ctx.link().send_future(async {
                    gloo::timers::future::sleep(Duration::from_secs(3)).await;
                    AppMsg::TurnOffShow
                });
                true
            }
            AppMsg::Fetching => false,
            AppMsg::TurnOffShow => {
                self.message = None;
                self.error = None;
                self.show_error = false;
                self.show_message = false;
                true
            }
            AppMsg::RunCode => {
                let code = self.code.to_string();

                ctx.link().send_future(async move {
                    match send_code(URL, code).await {
                        Ok(ret) if !ret.is_empty() => AppMsg::Response(ret),
                        Err(e) => AppMsg::ResponseError(e),
                        _ => AppMsg::ResponseError(SendError::new("Status error")),
                    }
                });

                ctx.link().send_message(AppMsg::Fetching);
                false
            }
            AppMsg::CodeChanged(code) => {
                self.code = code;
                false
            }
            AppMsg::ResponseError(e) => {
                self.error = Some(e.err);
                self.show_error = true;
                ctx.link().send_future(async {
                    gloo::timers::future::sleep(Duration::from_secs(3)).await;
                    AppMsg::TurnOffShow
                });
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let App {
            code: _code,
            message,
            error,
            show_error,
            show_message,
        } = self;
        let message = if *show_message { message.clone() } else { None };
        let error = if *show_error { error.clone() } else { None };

        let run_code = ctx.link().callback(|_| AppMsg::RunCode);

        let code_on_changed = ctx.link().callback(|e: Event| {
            AppMsg::CodeChanged(e.target_unchecked_into::<HtmlInputElement>().value())
        });
        html! {
            <div class="body_div">
                <div class="navbar bg-base-100">
                    <div class="navbar-start"/>
                    <div class="navbar-center">
                        <a class="btn btn-ghost normal-case text-xl">{"Monkey Language"}</a>
                    </div>
                    <div class="navbar-end"/>
                </div>
                <div
                    class="flex flex-col w-full lg:flex-row"
                    style="padding: 20px; height: 90%;"
                >
                    <div class="grid flex-grow  card bg-base-300 rounded-box place-items-center" >
                        <textarea
                            class="textarea from-control"
                            style="width: 95%; height:95%; resize: none;"
                            placeholder={PLACEHOLDER}
                            onchange={code_on_changed}
                        />
                    </div>
                    <div class="divider lg:divider-horizontal">{" "}</div>
                    <div
                        class="flex flex-col"
                        style="width: 35%;"
                    >
                        <button class="btn btn-outline" onclick={run_code}>{"Run the Code"}</button>
                        <div class="divider"></div>
                        <div class="grid flex-grow  card bg-base-300 rounded-box place-items-center">
                            <div class="message_board">{EXAMPLE_CODE}</div>
                        </div>
                    </div>
                </div>
                if *show_message {
                    <div class="alert alert-info shadow-lg">
                        <div>
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current flex-shrink-0 w-6 h-6"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                            <span>
                                {message.unwrap()}
                            </span>
                        </div>
                    </div>
                }

                if *show_error {
                    <div class="alert alert-error shadow-lg">
                        <div>
                            <svg xmlns="http://www.w3.org/2000/svg" class="stroke-current flex-shrink-0 h-6 w-6" fill="none" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                            <span>
                                {error.unwrap()}
                            </span>
                        </div>
                    </div>
                }
            </div>
        }
    }
}

const PLACEHOLDER: &str = r#"  
	let identity = fn(x) { x; }; identity(5);
    "#;
const EXAMPLE_CODE: &str = r#"
    Code:
	let identity = fn(x) { x; }; identity(5);
    Response: 5

    Code:
	let identity = fn(x) { return x; }; identity(5);
    Response: 5
    
    Code: 
	let double = fn(x) { x * 2; }; double(5);
    Response: 10

    Code:
	let add = fn(x, y) { x + y; }; add(5, 5);
    Response: 10

    Code:
	let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));
    Response: 20

    Code:
	fn(x) { x; }(5)
    Response: 5
"#;
