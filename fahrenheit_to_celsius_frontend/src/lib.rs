use wasm_bindgen::prelude::*;
use yew::prelude::*;
use web_sys::HtmlInputElement;
use gloo_net::http::Request;

#[function_component(App)]
fn app() -> Html {
    let fahrenheit = use_state(|| String::new());
    let celsius = use_state(|| None);

    let oninput = {
        let fahrenheit = fahrenheit.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            fahrenheit.set(input.value());
        })
    };

    let onclick = {
        let fahrenheit = fahrenheit.clone();
        let celsius = celsius.clone(); // Clone the state to keep ownership
        Callback::from(move |_| {
            let fahrenheit_value = fahrenheit.clone(); // Clone the fahrenheit state
            let celsius = celsius.clone(); // Clone to avoid moving
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(f) = fahrenheit_value.parse::<f64>() {
                    let response = Request::get(&format!("/convert?fahrenheit={}", f))
                        .send()
                        .await
                        .unwrap()
                        .json::<serde_json::Value>()
                        .await
                        .unwrap();
                    let c = response["celsius"].as_f64().unwrap();
                    celsius.set(Some(c)); // Update the celsius state
                }
            });
        })
    };


    html! {
        <div>
            <input {oninput} type="number" placeholder="Enter Fahrenheit" />
            <button {onclick}>{ "Convert" }</button>
            {
                if let Some(celsius) = *celsius {
                    html! { <p>{ format!("Celsius: {:.2}", celsius) }</p> }
                } else {
                    html! { <p>{ "Enter a value to convert" }</p> }
                }
            }
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
