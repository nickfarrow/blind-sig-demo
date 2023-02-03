mod utils;

use rand::rngs::ThreadRng;
use schnorr_fun::{
    blind::{self, Blinder},
    nonce, Message, Schnorr,
};
use sha2::Sha256;
use wasm_bindgen::prelude::*;

use gloo::events::EventListener;
use wasm_bindgen::JsCast;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("WASM probably loaded if you're seeing this");
}

#[wasm_bindgen]
pub struct BlinderInterface {
    blinder_session: Blinder,
}

#[wasm_bindgen]
impl BlinderInterface {
    #[wasm_bindgen(constructor)]
    pub fn new(pub_nonce: String, server_pubkey: String, message: String) -> BlinderInterface {
        let nonce_gen = nonce::Synthetic::<Sha256, nonce::GlobalRng<ThreadRng>>::default();
        let user_schnorr = Schnorr::<Sha256, _>::new(nonce_gen.clone());

        let session = blind::Blinder::blind(
            serde_json::from_str(&pub_nonce).expect("valid pub_nonce"),
            serde_json::from_str(&server_pubkey).expect("valid pub_nonce"),
            Message::raw(serde_json::from_str(&message).expect("valid message")),
            user_schnorr,
            &mut rand::thread_rng(),
        );
        BlinderInterface {
            blinder_session: session,
        }
    }

    pub fn create_signature_request(&self) -> String {
        serde_json::to_string(&self.blinder_session.signature_request()).unwrap()
    }

    pub fn get_blinding_tweaks(&self) -> String {
        serde_json::to_string(&self.blinder_session.blinding_tweaks).unwrap()
    }

    pub fn get_blinded_nonce(&self) -> String {
        serde_json::to_string(&self.blinder_session.blinded_nonce).unwrap()
    }

    pub fn get_challenge(&self) -> String {
        serde_json::to_string(&self.blinder_session.challenge).unwrap()
    }

    pub fn unblind(&self, blind_signature: String) -> String {
        serde_json::to_string(
            &self
                .blinder_session
                .unblind(serde_json::from_str(&blind_signature).expect("valid blind signature")),
        )
        .unwrap()
    }
}

// #[wasm_bindgen(start)]
#[wasm_bindgen]
pub fn gen_nonce() -> Result<(), JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p")?;
    val.set_text_content(Some("Hello from Rust!"));

    body.append_child(&val)?;

    Ok(())
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    let body = document
        .body()
        .expect("document expect to have have a body");

    let nonce_button = document
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .map_err(|_| ())
        .unwrap();
    nonce_button.set_inner_html("Click this button to generate a nonce");
    let on_down = EventListener::new(&nonce_button, "mousedown", move |_event| {
        web_sys::console::log_1(&"Generate nonce mousedown".into());
        // Get nonce
    });
    on_down.forget();
    body.append_child(&nonce_button).unwrap();

    Ok(())
}
