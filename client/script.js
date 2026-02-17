const API_URL = "";

function copyField(el) {
    var text = el.innerText.trim();
    if (!text) return;
    navigator.clipboard.writeText(text).then(function () {
        el.classList.add("copied");
        setTimeout(function () {
            el.classList.remove("copied");
        }, 1500);
    });
}

function request_nonce() {
    fetch(API_URL + "/nonce")
        .then((response) => response.json())
        .then(function (data) {
            console.log(data);
            document.getElementById("nonce").innerHTML = data.public_nonce;
            document.getElementById("usednonce_signform").innerHTML =
            data.public_nonce;
            document.getElementById("server_pubkey").innerHTML = data.server_pubkey;
            document.getElementById("enter-message-div").style.visibility = "visible";
        }).catch((data) => {
            document.getElementById("server_pubkey").innerHTML = "AHh the blind signing server is down :(";
        });
    return false;
}

function to_hex(our_string) {
    var result = "";
    for (i=0; i<our_string.length; i++) {
        hex = our_string.charCodeAt(i).toString(16);
        result += ("000"+hex).slice(-4);
    }
    return result;
}

function use_message() {
    var message = document.getElementById("message_input").value;
    document.getElementById("original_message").innerHTML = message;
    message = to_hex(message);
    document.getElementById("message").innerHTML = message;
    document.getElementById("create-blindings-div").style.visibility = "visible";
    document.getElementById("create-nostr-wasm-button").style.display = "inline";
    document.getElementById("unsigned-nostr-event").style.display = "none";
    return false;
}

function hit_apply_bindings() {
    document.getElementById("blinded_values").style.display = "inline";
    document.getElementById("sign-challenge-div").style.visibility = "visible";

    document.getElementById("message_verifyform").innerHTML = document.getElementById("message").innerHTML;
    document.getElementById("pubkey_verifyform").innerHTML = document.getElementById("server_pubkey").innerHTML;
    document.getElementById("blinded_nonce_verifyform").innerHTML = document.getElementById("blinded_nonce").innerHTML;
    return false;
}

function request_sign() {
    fetch(
        API_URL + "/sign?" +
            new URLSearchParams({
                public_nonce: document.getElementById("nonce").innerHTML,
                challenge: document.getElementById("blinded_challenge").innerHTML,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            if (data.signature) {
                document.getElementById("blinded_signature").innerHTML = data.signature;
                document.getElementById("unblind-signature-div").style.visibility = "visible";
            } else {
                document.getElementById("blinded_signature").innerHTML = "SIGNING SERVER REFUSED TO SIGN! Nonce expired (be fast), or are you trying to reuse this nonce?"
            }
        });
    return false;
}

function request_verify() {
    fetch(
        API_URL + "/verify?" +
            new URLSearchParams({
                message: document.getElementById("message_verifyform").innerHTML,
                signature: document.getElementById("signature_verifyform").value,
                public_nonce: document.getElementById("blinded_nonce_verifyform").innerHTML,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            if (data.valid) {
                document.getElementById("verify_success").innerHTML = "Valid signature!";
            } else {
                document.getElementById("verify_success").innerHTML = "INVALID SIGNATURE";
            }
        });
    return false;
}
