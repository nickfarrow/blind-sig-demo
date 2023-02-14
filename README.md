# Blind Schorr Signature Demo

Disguise a Nostr event, blindly sign, then unblind the signature and post it!

## [Live demo](https://blindsigs.utxo.club)

## What are blind signatures?

In order to sign a message with regular schnorr signatures, a secret key and a nonce (random number) are used to sign a challenge for the message, producing a signature. This signature-nonce pair can then be verified against the signer's public key, proving the person signed off on the message using their secret key.

Blind signatures allow for a user (you!) to disguise a message challenge and then request a server to sign your disguised challenge. The user can then take this signature and unblind it. Resulting in a signature that is valid for the original challenge they disguised, while looking completely different to the challenge which the server was asked to sign.

Blind signatures are powerful for protecting user privacy, where a server can act as an authority or coordinator while remaining blind to the exact details of what they are signing off on.

See this [fantastic article](https://suredbits.com/schnorr-applications-blind-signatures/) by Nadav @ suredbits for an intro on blind signatures and some motivations for the mathematics.

## Running locally

### Build the WASM Client

```
cd client/
wasm-pack build --target web
cd ..
```

### Run the Website & Signing Server

In the project root directory, run:

```
cargo run
```

The server will generate a random secret (if it does not already exist) and write it to `secret`.

The port can be configured by altering `Rocket.toml`
