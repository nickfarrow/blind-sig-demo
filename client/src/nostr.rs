use schnorr_fun::{
    fun::{marker::EvenY, Point},
    Signature,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use web_sys::WebSocket;

#[derive(Serialize, Deserialize)]
pub struct UnsignedEvent {
    pub id: String,
    pubkey: Point<EvenY>,
    created_at: u64,
    kind: u64,
    tags: Vec<Vec<String>>,
    content: String,
    //hacky and gross
    // Should be Message::raw(unsigned_frostr_event.hash_bytes.as_slice());
    pub hash_bytes: Vec<u8>,
}

impl UnsignedEvent {
    pub fn new_unsigned(
        pubkey: Point<EvenY>,
        kind: u64,
        tags: Vec<Vec<String>>,
        content: String,
        created_at: u64,
    ) -> Self {
        let serialized_event = json!([0, pubkey, created_at, kind, json!(tags), content]);
        println!(
            "This is the FROSTR event to be created: {}\n",
            &serialized_event
        );

        let mut hash = Sha256::default();
        hash.update(serialized_event.to_string().as_bytes());
        let hash_result = hash.finalize();
        let hash_result_str = format!("{:x}", hash_result);

        Self {
            id: hash_result_str,
            pubkey,
            created_at,
            kind,
            tags,
            content,
            hash_bytes: hash_result.to_vec(),
        }
    }

    pub fn add_signature(self, signature: Signature) -> SignedEvent {
        SignedEvent {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig: signature,
        }
    }
}

#[derive(Serialize)]
pub struct SignedEvent {
    pub id: String,
    pubkey: Point<EvenY>,
    created_at: u64,
    kind: u64,
    tags: Vec<Vec<String>>,
    content: String,
    sig: Signature,
}

// Adapted from https://github.com/rot13maxi/moe-bot/
fn publish_to_relay(relay: &str, message: &str) -> Result<(), String> {
    let ws = WebSocket::new(relay).unwrap();
    ws.send_with_str(message).unwrap_or_else(|_| ());
    Ok(())
}

// Adapted from https://github.com/rot13maxi/moe-bot/
pub fn broadcast_event(event: &SignedEvent) {
    let event_json = json!(event).to_string();
    dbg!("{}", &event_json);

    let event_msg = json!(["EVENT", event]).to_string();
    dbg!("{}", &event_msg);

    for relay in vec![
        // "wss://relay.damus.io",
        "wss://nostr.zebedee.cloud",
        "wss://relay.nostr.ch",
        "wss://nostr-pub.wellorder.net",
        "wss://nostr-pub.semisol.dev",
        "wss://nostr.oxtr.dev",
    ] {
        match publish_to_relay(relay, &event_msg) {
            Ok(_) => println!("sent message to {}", relay),
            Err(e) => eprintln!("{}", e),
        };
    }
}
