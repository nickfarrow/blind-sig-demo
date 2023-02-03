#[macro_use]
extern crate rocket;

use blind_sig_demo::cors;
use rand::RngCore;
use rocket::serde::{json::Json, Serialize};
use rocket::{get, launch, routes, State};
use schnorr_fun::blind::{BlindSigner, SignatureRequest};
use schnorr_fun::nonce::GlobalRng;
use schnorr_fun::nonce::Synthetic;
use schnorr_fun::{Message, Signature};
use std::str::FromStr;
use std::sync::Mutex;

use rand::rngs::ThreadRng;
use schnorr_fun::fun::Point;
use schnorr_fun::{
    fun::{marker::*, Scalar},
    Schnorr,
};
use sha2::Sha256;

pub struct BlindSignerState {
    state: Mutex<BlindSigner<Sha256, Synthetic<Sha256, GlobalRng<ThreadRng>>>>,
}

#[derive(Serialize)]
pub struct NonceResponse {
    public_nonce: Point<EvenY>,
    server_pubkey: Point<EvenY>,
}

#[get("/nonce")]
pub fn nonce(signer_state: &State<BlindSignerState>) -> Json<NonceResponse> {
    let mut blind_signer = signer_state.inner().state.lock().unwrap();

    // Random session id
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 64];
    rng.fill_bytes(&mut bytes);

    let public_nonce = blind_signer.gen_nonce(&bytes);
    Json(NonceResponse {
        public_nonce,
        server_pubkey: blind_signer.public_key(),
    })
}

#[derive(Serialize)]
pub struct SignatureResponse {
    signature: Scalar<Public, Zero>,
}

#[get("/sign?<public_nonce>&<challenge>")]
pub async fn sign(
    signer_state: &State<BlindSignerState>,
    public_nonce: String,
    challenge: String,
) -> Json<SignatureResponse> {
    let signature_request = SignatureRequest {
        public_nonce: Point::from_str(&public_nonce).unwrap(),
        blind_challenge: Scalar::from_str(&challenge).unwrap(),
    };

    let mut blind_signer = signer_state.inner().state.lock().unwrap();
    // Try sign the request
    let _signature_response = blind_signer.sign(signature_request.clone(), &mut rand::thread_rng());
    let signature = loop {
        let has_response = blind_signer.lookup_signed(signature_request.public_nonce);

        match has_response {
            None => {
                // pause then poll again
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Some(response) => {
                // We have received some response
                match response {
                    Some(sig) => break sig,
                    None => panic!(), //TODO gently kill the disconnect
                };
            }
        }
    };

    Json(SignatureResponse { signature })
}

#[derive(Serialize)]
pub struct VerifyResponse {
    valid: bool,
}

#[get("/verify?<message>&<signature>&<public_nonce>")]
pub fn verify(
    message: String,
    signature: String,
    public_nonce: String,
    signer_state: &State<BlindSignerState>,
) -> Json<VerifyResponse> {
    let signature_scalar = match Scalar::from_str(&signature) {
        Ok(sig) => sig,
        Err(_) => return Json(VerifyResponse { valid: false }),
    };
    let blinded_pubnonce: Point<Normal> =
        Point::from_str(&public_nonce).expect("valid formed public nonce");

    let signature: Signature<Public> = Signature {
        s: signature_scalar,
        R: blinded_pubnonce.into_point_with_even_y().0,
    };

    let blind_signer = signer_state.inner().state.lock().unwrap();
    Json(VerifyResponse {
        valid: blind_signer.schnorr.verify(
            &blind_signer.public_key(),
            Message::<Public>::raw(message.as_bytes()),
            &signature,
        ),
    })
}

#[launch]
fn rocket() -> _ {
    let nonce_gen = Synthetic::<Sha256, GlobalRng<ThreadRng>>::default();
    let server_schnorr = Schnorr::<Sha256, _>::new(nonce_gen);
    let secret = Scalar::random(&mut rand::thread_rng());
    let n_sessions = 1;

    let blind_signer = BlindSigner::new(n_sessions, secret, server_schnorr);

    rocket::build()
        .mount("/", routes![nonce, sign, verify])
        .attach(cors::CORS)
        .manage(BlindSignerState {
            state: Mutex::new(blind_signer),
        })
}
