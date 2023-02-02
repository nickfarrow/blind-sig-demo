
// function hit_genblind() {
//     var nonce = document.getElementById("nonce").innerHTML;
//     var server_pubkey = document.getElementById("server_pubkey").innerHTML;
//     var message = document.getElementById("message").innerHTML;

//     var _interface = wasm.BlinderInterface.new(nonce, server_pubkey, message);
// }

// const API_URL = "http://127.0.0.1:8000";

// function gen_nonce() {
//     fetch(API_URL + "/nonce")
//         .then((response) => response.json())
//         .then(function (data) {
//             console.log(data);
//             document.getElementById("nonce").innerHTML = data.public_nonce;
//             document.getElementById("usednonce_signform").value =
//             data.public_nonce;
//             document.getElementById("server_pubkey").innerHTML = data.server_pubkey;
//         });
//     return false;
// }


// function request_sign() {
//     fetch(
//         API_URL + "/sign?" +
//             new URLSearchParams({
//                 nonce: document.getElementById("public_nonce").innerHTML,
//                 message: document.getElementById("blind_challenge").value,
//             })
//     )
//         .then((response) => response.json())
//         .then(function (data) {

//         });
//     return false;
// }
