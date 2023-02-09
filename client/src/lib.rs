mod nostr;
mod utils;
use crate::utils::set_panic_hook;

use std::str::FromStr;

use nostr::UnsignedEvent;
use rand::rngs::ThreadRng;
use schnorr_fun::{
    blind::{self, Blinder},
    fun::{
        marker::{Normal, Public},
        Point, Scalar,
    },
    nonce, Message, Schnorr, Signature,
};
use sha2::Sha256;
use wasm_bindgen::prelude::*;

use gloo::events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::Window;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
}

#[wasm_bindgen]
pub fn greet() {
    alert("Sucessfully loaded secp256kfun Schnorr Blind Signature WASM 🦀");
}

fn create_gen_blindings_button(window: &Window) {
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
                "Success reading nonce ({nonce_input}), pubkey ({server_pubkey_input}), message ({message_text}) into rust",
            )
            .into(),
        );

        let nonce = Point::from_str(&nonce_input).expect("valid nonce");
        let pubkey = Point::from_str(&server_pubkey_input).expect("valid pubkey");
        let message_bytes = hex::decode(message_text).unwrap();
        let message = Message::raw(&message_bytes);

        let nonce_gen = nonce::Synthetic::<Sha256, nonce::GlobalRng<ThreadRng>>::default();
        let schnorr = Schnorr::<Sha256, _>::new(nonce_gen);

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
}

fn create_to_nostr_message_button(window: &Window) {
    // Create a to-nostr button
    let document = window.document().expect("expecting a document on window");
    let to_nostr_message_button = document
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .map_err(|_| ())
        .unwrap();
    to_nostr_message_button.set_inner_html("Convert into Nostr event");
    let on_down = EventListener::new(&to_nostr_message_button, "mousedown", move |_event| {
        web_sys::console::log_1(&"Transform into Nostr Event".into());

        let existing_message = document
            .get_element_by_id("original_message")
            .unwrap()
            .inner_html();

        let server_pubkey_input = document
            .get_element_by_id("server_pubkey")
            .unwrap()
            .inner_html();

        let pubkey = Point::from_str(&server_pubkey_input).expect("valid pubkey");

        // Cant use std::time (panic)
        // https://github.com/rust-lang/rust/issues/48564
        let time_now = date_now() as u64 / 1000;

        let unsigned_event =
            nostr::UnsignedEvent::new_unsigned(pubkey, 1, Vec::new(), existing_message, time_now);
        web_sys::console::log_1(&"Writing message".into());

        document
            .get_element_by_id("message")
            .unwrap()
            .set_inner_html(&unsigned_event.id);

        // Store unsigned nostr event for later broadcasting
        document
            .get_element_by_id("nostr_event")
            .unwrap()
            .set_inner_html(&serde_json::to_string(&unsigned_event).unwrap());

        // Hide the button again
        document
            .get_element_by_id("create-nostr-wasm-button")
            .unwrap()
            .set_attribute("style", "display: none;") // Hacky -- idk how to properly set style.visibility
            .unwrap();
        document
            .get_element_by_id("unsigned-nostr-event")
            .unwrap()
            .set_attribute("style", "") // Hacky -- idk how to properly set style.visibility
            .unwrap();
        // Unhide the nostr broadcast div
        document
            .get_element_by_id("broadcast-nostr-div")
            .unwrap()
            .set_attribute("style", "") // Hacky -- idk how to properly set style.visibility
            .unwrap();
        web_sys::console::log_1(&"Wrote nostr event messages".into());
    });
    on_down.forget();

    // Write to_nostr_message_button into HTML
    let document = window.document().expect("expecting a document on window");
    document
        .get_element_by_id("create-nostr-wasm-button")
        .unwrap()
        .append_child(&to_nostr_message_button)
        .unwrap();
}

fn create_unblind_button(window: &Window) {
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
}

fn create_broadcast_nostr_button(window: &Window) {
    // Create a unblind button
    let document = window.document().expect("expecting a document on window");
    let broadcast_button = document
        .create_element("button")
        .unwrap()
        .dyn_into::<web_sys::HtmlButtonElement>()
        .map_err(|_| ())
        .unwrap();
    broadcast_button.set_inner_html("Add signature & broadcast event!");
    let on_down = EventListener::new(&broadcast_button, "mousedown", move |_event| {
        web_sys::console::log_1(&"Broadcasting nostr event".into());

        let nostr_unsigned: UnsignedEvent = serde_json::from_str(
            &document
                .get_element_by_id("nostr_event")
                .unwrap()
                .inner_html(),
        )
        .unwrap();

        web_sys::console::log_1(&"Read in nostr event".into());

        let blinded_nonce = document
            .get_element_by_id("blinded_nonce")
            .unwrap()
            .inner_html();
        let signature = document
            .get_element_by_id("unblinded_signature")
            .unwrap()
            .inner_html();

        let blinded_pubnonce: Point<Normal> =
            Point::from_str(&blinded_nonce).expect("valid formed public nonce");

        let signature: Signature<Public> = Signature {
            s: Scalar::from_str(&signature).unwrap(),
            R: blinded_pubnonce.into_point_with_even_y().0,
        };
        let nostr_signed = nostr_unsigned.add_signature(signature);

        document
            .get_element_by_id("signed_nostr_event")
            .unwrap()
            .set_inner_html(&serde_json::to_string(&nostr_signed).unwrap());

        web_sys::console::log_1(&"Attached signature to event!".into());

        nostr::broadcast_event(&nostr_signed);
        alert(&format!("Broadcasted Nostr event: {} . Search on nostr.guru, sometimes takes a while to appear..", nostr_signed.id));
        web_sys::console::log_1(&"Broadcasted nostr event!".into());
    });
    on_down.forget();

    // Write unblind_button into HTML
    let document = window.document().expect("expecting a document on window");
    document
        .get_element_by_id("broadcast-nostr-button-wasm")
        .unwrap()
        .append_child(&broadcast_button)
        .unwrap();
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    set_panic_hook();
    let window = web_sys::window().expect("global window does not exists");

    create_gen_blindings_button(&window);
    create_to_nostr_message_button(&window);
    create_unblind_button(&window);
    create_broadcast_nostr_button(&window);

    Ok(())
}
