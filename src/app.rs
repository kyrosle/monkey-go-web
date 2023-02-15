use std::{
    error::Error,
    fmt::{Debug, Display},
};

use gloo_console::log;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{HtmlInputElement, Request, RequestInit, Response};
use yew::prelude::*;

const URL: &str = "http://localhost:8888/code";

#[derive(Debug, Clone, PartialEq)]
pub struct SendError {
    err: JsValue,
}

impl Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.err, f)
    }
}
impl Error for SendError {}

impl From<JsValue> for SendError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}

pub struct App {
    code: String,
    message: Option<String>,
}

pub enum AppMsg {
    Response(String),
    ResponseError(SendError),
    RunCode,
    Fetching,
    CodeChanged(String),
}

async fn send_code(url: &'static str, value: String) -> Result<String, SendError> {
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(web_sys::RequestMode::NoCors);
    opts.body(Some(&JsValue::from_str(&value)));

    let request = Request::new_with_str_and_init(url, &opts)?;

    let window = gloo::utils::window();

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();
    log!("", resp.status());

    let text = JsFuture::from(resp.json()?).await?;

    Ok(text.as_string().unwrap())
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App {
            code: PLACEHOLDER.to_string(),
            message: Some("Info".to_string()),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::Response(ret) => {
                log!("Response: ", &ret);
                self.message = Some(ret);
                true
            }
            AppMsg::Fetching => false,
            AppMsg::RunCode => {
                let code = self.code.to_string();

                ctx.link().send_future(async move {
                    match send_code(URL, code).await {
                        Ok(ret) => AppMsg::Response(ret),
                        Err(_) => AppMsg::RunCode,
                    }
                });

                ctx.link().send_message(AppMsg::Fetching);
                false
            }
            AppMsg::CodeChanged(code) => {
                self.code = code;
                false
            }
            AppMsg::ResponseError(_) => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let App {
            code: _code,
            message,
        } = self;
        let info_active = message.is_some();
        let message = if info_active { message.clone() } else { None };

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
                if info_active {
                    <div class="alert alert-info shadow-lg">
                        <div>
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current flex-shrink-0 w-6 h-6"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                            <span>
                                {message.unwrap()}
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
