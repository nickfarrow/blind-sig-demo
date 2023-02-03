const API_URL = "http://127.0.0.1:1234";

function gen_nonce() {
    fetch(API_URL + "/nonce")
        .then((response) => response.json())
        .then(function (data) {
            console.log(data);
            document.getElementById("nonce").innerHTML = data.public_nonce;
            document.getElementById("usednonce_signform").innerHTML =
            data.public_nonce;
            document.getElementById("server_pubkey").innerHTML = data.server_pubkey;
        });
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
            document.getElementById("blinded_signature").innerHTML = data.signature;
        });
    return false;
}

function use_message() {
    var message = document.getElementById("message_input").value;
    document.getElementById("message").innerHTML = message;
    return false;
}

function hit_apply_bindings() {
    document.getElementById("blinded_values").style.display = "inline";
    return false;
}
