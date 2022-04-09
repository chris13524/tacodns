#[macro_use]
extern crate rocket;

use anyhow::{anyhow, Result};
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use rocket::State as RState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Challenge {
    fqdn: String,
    value: String,
}

struct State {
    challenges: RwLock<HashMap<String, String>>,
}

fn get_impl(state: &RState<State>, path: PathBuf) -> Result<Json<Vec<String>>> {
    let mut path = path
        .iter()
        .map(|os_str| os_str.to_str())
        .collect::<Option<Vec<&str>>>()
        .ok_or_else(|| anyhow!("path contains non-unicode char"))?;
    trace!("path: {:?}", path);

    let record_type = path.pop().ok_or_else(|| anyhow!("expected record type"))?;
    if record_type != "TXT" {
        return Err(anyhow!("record_type != TXT:  {}", record_type));
    }

    let fqdn = format!(
        "{}.",
        path.into_iter().rev().collect::<Vec<&str>>().join(".")
    );
    trace!("fqdn: {}", fqdn);
    let challenge = state
        .challenges
        .read()
        .map_err(|e| anyhow!("expected challenges to lock: {:?}", e))?
        .get(&fqdn)
        .ok_or_else(|| anyhow!("expected challenge under: {}", fqdn))?
        .clone();
    Ok(Json(vec![challenge]))
}

#[get("/<path..>")]
fn get(state: &RState<State>, path: PathBuf) -> Result<Json<Vec<String>>, Status> {
    get_impl(state, path).map_err(|e| {
        error!("{:?}", e);
        Status::InternalServerError
    })
}

#[post("/present", data = "<challenge>")]
fn present(state: &RState<State>, challenge: Json<Challenge>) {
    let Challenge { fqdn, value } = challenge.0;
    trace!("present(challenge.fqdn:{})", fqdn);
    state.challenges.write().unwrap().insert(fqdn, value);
}

#[post("/cleanup", data = "<challenge>")]
fn cleanup(state: &RState<State>, challenge: Json<Challenge>) {
    let Challenge { fqdn, .. } = challenge.0;
    trace!("cleanup(challenge.fqdn:{})", fqdn);
    state.challenges.write().unwrap().remove(&fqdn);
}

#[launch]
fn rocket() -> _ {
    env_logger::init();
    rocket::build()
        .manage(State {
            challenges: RwLock::new(HashMap::new()),
        })
        .mount("/", routes![get, present, cleanup])
}
