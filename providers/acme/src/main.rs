#[macro_use]
extern crate rocket;

use rocket::response::status::NotFound;
use rocket::serde::{json::Json, Deserialize};
use rocket::State as RState;
use std::collections::HashMap;
use std::sync::Mutex;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Challenge {
    fqdn: String,
    value: String,
}

struct State {
    challenges: Mutex<HashMap<String, Challenge>>,
}

#[get("/get/<path..>/TXT")]
fn get(state: &RState<State>, path: PathBuf) -> Json<Vec<String>> {
    let fqdn = path.join(".");
    vec![state.challenges.lock().unwrap().get(fqdn)]
}

#[post("/present", data = "<challenge>")]
fn set(state: &RState<State>, challenge: Json<Challenge>) {
    state.challenges.lock().unwrap().insert(challenge.fqdn, *challenge);
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(State {
            challenges: Mutex::new(HashMap::new()),
        })
        .mount("/", routes![get, set])
}
