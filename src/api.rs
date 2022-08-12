use std::fs;
use std::path::Path;
use std::str::FromStr;

use rand::thread_rng;
use rocket::get;
use rocket::http::Status;
use rocket::serde::{json::Json, Serialize};
use schnorr_fun::{blind_sign, unblind_signature, Blinder, Signature, SignatureRequest};

use rand::rngs::ThreadRng;
use schnorr_fun::fun::{derive_nonce, g, Point, G};
use schnorr_fun::{
    fun::{marker::*, nonce, Scalar},
    Message, Schnorr,
};
use sha2::Sha256;

use rocket_db_pools::sqlx::sqlite::SqliteQueryResult;
use rocket_db_pools::sqlx::Acquire;
use rocket_db_pools::sqlx::SqliteConnection;
use rocket_db_pools::sqlx::{self, Row};
use rocket_db_pools::Connection;
use rocket_db_pools::Database;

#[derive(Database)]
#[database("main")]
pub struct Main(sqlx::SqlitePool);

fn get_sec_pub() -> (Scalar<Secret, NonZero>, Point<EvenY>) {
    let secret_file = "secret";
    if !Path::new(secret_file).exists() {
        println!("No existing secret found, creating secret.");
        let mut rng = thread_rng();
        let secret = Scalar::<Secret, NonZero>::random(&mut rng);
        fs::write(secret_file, secret.to_string()).expect("Unable to write file");
    }
    let mut secret = Scalar::<Secret, NonZero>::from_str(
        &fs::read_to_string(secret_file).expect("secret exists"),
    )
    .expect("valid");
    let (public_key, secret_needs_negation) = g!(secret * G).normalize().into_point_with_even_y();
    secret.conditional_negate(secret_needs_negation);
    (secret, public_key)
}

#[derive(Debug)]
enum NonceError {
    AlreadyUsed,
}

async fn save_nonce(conn: &mut SqliteConnection, secret_nonce: String, public_nonce: String) {
    sqlx::query("INSERT INTO nonces VALUES (?, ?)")
        .bind(secret_nonce)
        .bind(public_nonce)
        .execute(conn)
        .await
        .expect("stored nonce ok");
}

async fn get_nonce_secret(
    conn: &mut SqliteConnection,
    public_nonce: String,
) -> Result<String, NonceError> {
    let row_res = sqlx::query("SELECT * FROM nonces WHERE public_nonce=?")
        .bind(public_nonce.clone())
        .fetch_one(conn)
        .await;
    match row_res {
        Err(_) => Err(NonceError::AlreadyUsed),
        Ok(row) => {
            let secret_nonce: String = row.try_get(0).expect("some id");
            Ok(secret_nonce)
        }
    }
}

async fn delete_nonce_secret(
    conn: &mut SqliteConnection,
    public_nonce: String,
) -> Result<SqliteQueryResult, rocket_db_pools::sqlx::Error> {
    sqlx::query("DELETE FROM nonces WHERE public_nonce=?")
        .bind(public_nonce)
        .execute(conn)
        .await
}

#[derive(Serialize, Debug)]
pub struct NonceResponse {
    nonce: String,
}

#[get("/gennonce")]
pub async fn gennonce(mut pool: Connection<Main>) -> Json<NonceResponse> {
    let nonce_gen = nonce::Synthetic::<Sha256, nonce::GlobalRng<ThreadRng>>::default();
    let schnorr = Schnorr::<Sha256, _>::new(nonce_gen.clone());

    let mut nonce = derive_nonce!(
        nonce_gen => schnorr.nonce_gen(),
        secret => get_sec_pub().0,
        public => [ b"blind-signature"]
    );
    let (pub_nonce, nonce_negated) = g!(nonce * G).normalize().into_point_with_even_y();
    nonce.conditional_negate(nonce_negated);

    let secret_nonce_str = nonce.to_string();
    let public_nonce_str = pub_nonce.to_string();
    let mut txn = pool.begin().await.unwrap();
    save_nonce(&mut *txn, secret_nonce_str, public_nonce_str.clone()).await;
    txn.commit().await.unwrap();

    Json(NonceResponse {
        nonce: public_nonce_str,
    })
}

#[derive(Serialize)]
pub struct BlindSession {
    tweaked_pubkey: String,
    blinded_nonce: String,
    challenge: String,
    alpha: String,
    beta: String,
    t: String,
}

#[get("/blind?<message>&<nonce>")]
pub async fn blind(message: String, nonce: String) -> Json<BlindSession> {
    let nonce_gen = nonce::Synthetic::<Sha256, nonce::GlobalRng<ThreadRng>>::default();
    let schnorr = Schnorr::<Sha256, _>::new(nonce_gen.clone());

    let typed_nonce = Point::<EvenY, Public, _>::from_str(&nonce).expect("valid nonce");
    let blind_session = Blinder::blind(
        typed_nonce,
        get_sec_pub().1,
        Message::<Public>::plain("blind-schnorr", message.as_bytes()),
        schnorr,
        &mut rand::thread_rng(),
    );

    Json(BlindSession {
        tweaked_pubkey: blind_session.tweaked_pubkey.to_string(),
        blinded_nonce: blind_session.blinded_nonce.to_string(),
        challenge: blind_session.challenge.to_string(),
        alpha: blind_session.blinding_tweaks.alpha.to_string(),
        beta: blind_session.blinding_tweaks.beta.to_string(),
        t: blind_session.blinding_tweaks.t.to_string(),
    })
}

#[derive(Serialize, Debug)]
pub struct BlindSignature {
    signature: String,
    message: String,
}

#[get("/sign?<challenge>&<nonce>")]
pub async fn sign(
    mut pool: Connection<Main>,
    challenge: String,
    nonce: String,
) -> (Status, Json<BlindSignature>) {
    let signature_request = SignatureRequest {
        blind_challenge: Scalar::from_str(&challenge).expect("valid scalar"),
    };
    // Lookup nonce secret:
    let mut txn = pool.begin().await.unwrap();
    let secret = get_nonce_secret(&mut *txn, nonce.clone()).await;
    txn.commit().await.unwrap();

    match secret {
        Err(_) => {
            return (
                Status::InternalServerError,
                Json(BlindSignature {
                    signature: "".to_string(),
                    message: "server refuses to reuse this nonce".to_string(),
                }),
            )
        }
        Ok(secret) => {
            let mut txn = pool.begin().await.unwrap();
            let deleted = delete_nonce_secret(&mut *txn, nonce.clone()).await;
            txn.commit().await.unwrap();
            if deleted.is_err() {
                panic!("Failed to delete nonce from database!!");
            }

            let typed_secret = Scalar::<Secret, NonZero>::from_str(&secret).expect("valid scalar");
            let blind_signature = blind_sign(&get_sec_pub().0, typed_secret, signature_request);
            return (
                Status::Accepted,
                Json(BlindSignature {
                    signature: blind_signature.to_string(),
                    message: "ok".to_string(),
                }),
            );
        }
    }
}

#[derive(Serialize, Debug)]
pub struct UnblindedSignature {
    signature: String,
}

#[get("/unblind?<blindsig>&<alpha>&<challenge>&<tweak>")]
pub async fn unblind(
    blindsig: String,
    alpha: String,
    challenge: String,
    tweak: String,
) -> Json<UnblindedSignature> {
    let unblinded_signature = unblind_signature(
        Scalar::from_str(&blindsig).expect("valid scalar"),
        &Scalar::from_str(&alpha).expect("valid scalar"),
        &Scalar::from_str(&challenge).expect("valid scalar"),
        &Scalar::from_str(&tweak).expect("valid scalar"),
    );
    Json(UnblindedSignature {
        signature: unblinded_signature.to_string(),
    })
}

#[derive(Serialize, Debug)]
pub struct ValidResponse {
    valid: bool,
}

#[get("/verify?<signature>&<tweaked_pubkey>&<blinded_nonce>&<message>")]
pub async fn verify(
    signature: String,
    blinded_nonce: String,
    message: String,
    tweaked_pubkey: String,
) -> Json<ValidResponse> {
    let nonce_gen = nonce::Synthetic::<Sha256, nonce::GlobalRng<ThreadRng>>::default();
    let schnorr = Schnorr::<Sha256, _>::new(nonce_gen.clone());
    let (verification_pubkey, _) = Point::<Normal, Public, _>::from_str(&tweaked_pubkey)
        .expect("valid pubkey")
        .into_point_with_even_y();

    let sig_read = Scalar::from_str(&signature);

    Json(ValidResponse {
        valid: {
            if sig_read.is_err() {
                false
            } else {
                let signature = Signature {
                    s: sig_read.unwrap(),
                    R: Point::<Normal, Public, NonZero>::from_str(&blinded_nonce)
                        .expect("valid nonce")
                        .to_xonly(),
                };
                schnorr.verify(
                    &verification_pubkey,
                    Message::<Public>::plain("blind-schnorr", message.as_bytes()),
                    &signature,
                )
            }
        },
    })
}
