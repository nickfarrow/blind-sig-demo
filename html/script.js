globalThis.paused = true;
const timer = (ms) => new Promise((res) => setTimeout(res, ms));

function hide_applied_blindings() {
    document.getElementById("blinded_nonce").style.display = "none";
    document.getElementById("blinded_publickey").style.display = "none";
    document.getElementById("challenge").style.display = "none";
    document.getElementById("challenge_signform").style.value = "";

    return false;
}
hide_applied_blindings();

function gen_nonce() {
    fetch("/api/gennonce")
        .then((response) => response.json())
        .then(function (data) {
            console.log(data);
            document.getElementById("nonce").innerHTML = data.nonce;
            document.getElementById("usednonce_signform").value =
                data.nonce;
        });
    return false;
}

function reset_blind() {
    document.getElementById("blinded_nonce").innerHTML = "";
    document.getElementById("blinded_publickey").innerHTML = "";
    document.getElementById("challenge").innerHTML = "";
    document.getElementById("challenge_signform").value = "";
    document.getElementById("alpha").value = "";
    document.getElementById("beta").value = "";
    document.getElementById("t").value = "";
    hide_applied_blindings();
}

function hit_genblind() {
    fetch(
        "/api/blind?" +
            new URLSearchParams({
                nonce: document.getElementById("nonce").innerHTML,
                message: document.getElementById("message").value,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            document.getElementById("message_verifyform").value =
                document.getElementById("message").value;
            document.getElementById("blinded_pubkey_verifyform").value =
                data.tweaked_pubkey;
            document.getElementById("message").value;

            document.getElementById("blinded_nonce").innerHTML =
                data.blinded_nonce;
            document.getElementById("blinded_nonce_verifyform").value =
                data.blinded_nonce;
            document.getElementById("blinded_publickey").innerHTML =
                data.tweaked_pubkey;
            globalThis.challenge = data.challenge;
            document.getElementById("challenge").innerHTML =
                globalThis.challenge;
            document.getElementById("challenge_signform").value = "";
            document.getElementById("alpha").value = data.alpha;
            document.getElementById("beta").value = data.beta;
            document.getElementById("t").value = data.t;
        });
    return false;
}

function hit_apply_bindings() {
    document.getElementById("blinded_nonce").style.display = "inline";
    document.getElementById("blinded_publickey").style.display =
        "inline";
    document.getElementById("challenge").style.display = "inline";
    document.getElementById("challenge_signform").value =
        globalThis.challenge;
    return false;
}

function hit_sign() {
    fetch(
        "/api/sign?" +
            new URLSearchParams({
                nonce: document.getElementById("usednonce_signform")
                    .value,
                challenge:
                    document.getElementById("challenge_signform").value,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            if (data.message == "ok") {
                document.getElementById("blinded_signature").innerHTML =
                    data.signature;
            } else {
                document.getElementById("blinded_signature").innerHTML =
                    data.message;
            }
        });
    return false;
}

function hit_unblind() {
    fetch(
        "/api/unblind?" +
            new URLSearchParams({
                blindsig:
                    document.getElementById("blinded_signature")
                        .innerHTML,
                challenge:
                    document.getElementById("challenge_signform").value,
                alpha: document.getElementById("alpha").value,
                tweak: document.getElementById("t").value,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            // console.log(data);
            document.getElementById("unblinded_signature").innerHTML =
                data.signature;
        });
    return false;
}

function hit_verify() {
    fetch(
        "/api/verify?" +
            new URLSearchParams({
                tweaked_pubkey:
                    document.getElementById("blinded_publickey")
                        .innerHTML,
                message: document.getElementById("message").value,
                blinded_nonce: document.getElementById(
                    "blinded_nonce_verifyform"
                ).value,
                signature: document.getElementById(
                    "signature_verifyform"
                ).value,
            })
    )
        .then((response) => response.json())
        .then(function (data) {
            // console.log(data);
            if (data.valid) {
                document.getElementById("verify_success").innerHTML =
                    "Valid signature :)";
            } else {
                document.getElementById("verify_success").innerHTML =
                    "INVALID SIGNATURE";
            }
        });
    return false;
}
