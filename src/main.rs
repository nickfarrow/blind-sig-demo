#[macro_use]
extern crate rocket;

use blind_sig_demo::api::Main;
use blind_sig_demo::api::{blind, gennonce, sign, unblind, verify};
use rocket::fs::FileServer;
use rocket_db_pools::Database;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Main::init())
        .mount("/", FileServer::from("./html"))
        .mount("/api", routes![blind, gennonce, sign, unblind, verify])
}
