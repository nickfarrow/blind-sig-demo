mod utils;

use std::str::FromStr;

use rand::rngs::ThreadRng;
use schnorr_fun::{
    blind::{self, Blinder},
    fun::{Point, Scalar},
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

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");

    let gen_blindings_button = document
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .map_err(|_| ())
        .unwrap();
    gen_blindings_button.set_inner_html("Generate blinding values");
    let on_down = EventListener::new(&gen_blindings_button, "mousedown", move |_event| {
        web_sys::console::log_1(&"Generate blinding values".into());
        // Read nonces from doc
        let nonce_input = document.get_element_by_id("nonce").unwrap().inner_html();
        let server_pubkey_input = document
            .get_element_by_id("server_pubkey")
            .unwrap()
            .inner_html();
        let message_input = document.get_element_by_id("message").unwrap();

        let message_text = message_input.inner_html();
        web_sys::console::log_1(
            &format!(
                "Success reading nonce ({}), pubkey ({}), message ({}) into rust",
                nonce_input, server_pubkey_input, message_text
            )
            .into(),
        );
        let nonce = Point::from_str(&nonce_input).expect("valid nonce");
        let pubkey = Point::from_str(&server_pubkey_input).expect("valid pubkey");
        let message = Message::raw(message_text.as_bytes());

        let nonce_gen = nonce::Synthetic::<Sha256, nonce::GlobalRng<ThreadRng>>::default();
        let schnorr = Schnorr::<Sha256, _>::new(nonce_gen.clone());

        web_sys::console::log_1(&"About to apply blindings".into());

        // Generate blinding tweaks
        let blinder = Blinder::blind(nonce, pubkey, message, schnorr, &mut rand::thread_rng());

        // Store alpha and beta
        document
            .get_element_by_id("alpha")
            .unwrap()
            .set_inner_html(&blinder.blinding_tweaks.alpha.to_string());
        document
            .get_element_by_id("beta")
            .unwrap()
            .set_inner_html(&blinder.blinding_tweaks.beta.to_string());

        // Store the blinded nonce and challenge too:
        document
            .get_element_by_id("blinded_nonce")
            .unwrap()
            .set_inner_html(&blinder.blinded_nonce.to_string());
        document
            .get_element_by_id("blinded_challenge")
            .unwrap()
            .set_inner_html(&blinder.challenge.to_string());
        document
            .get_element_by_id("challenge_signform")
            .unwrap()
            .set_inner_html(&blinder.challenge.to_string());

        // Unhide the next div
        document
            .get_element_by_id("apply-blindings-div")
            .unwrap()
            .set_attribute("style", "") // Hacky -- idk how to properly set style.visibility
            .unwrap();

        web_sys::console::log_1(&format!("Blinder {:?}", blinder.blinding_tweaks).into());
    });
    on_down.forget();

    // Write generate tweaks button into HTML
    let document = window.document().expect("expecting a document on window");
    document
        .get_element_by_id("create-blindings-wasm-button")
        .unwrap()
        .append_child(&gen_blindings_button)
        .unwrap();

    // Create a unblind button
    let document = window.document().expect("expecting a document on window");
    let unblind_button = document
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .map_err(|_| ())
        .unwrap();
    unblind_button.set_inner_html("Unblind Signature");
    let on_down = EventListener::new(&unblind_button, "mousedown", move |_event| {
        web_sys::console::log_1(&"Unblind Signature".into());
        // Read nonces from doc
        let blinded_signature = Scalar::from_str(
            &document
                .get_element_by_id("blinded_signature")
                .unwrap()
                .inner_html(),
        )
        .expect("valid signature string");

        let alpha = Scalar::from_str(&document.get_element_by_id("alpha").unwrap().inner_html())
            .expect("valid alpha string");

        let signature = blind::unblind_signature(blinded_signature, &alpha);
        document
            .get_element_by_id("unblinded_signature")
            .unwrap()
            .set_inner_html(&signature.to_string());

        // Unhide the next div
        document
            .get_element_by_id("bottom-row-div")
            .unwrap()
            .set_attribute("style", "") // Hacky -- idk how to properly set style.visibility
            .unwrap();
    });
    on_down.forget();

    // Write unblind_button into HTML
    let document = window.document().expect("expecting a document on window");
    document
        .get_element_by_id("unblind-signature-wasm-button")
        .unwrap()
        .append_child(&unblind_button)
        .unwrap();

    Ok(())
}
