#[macro_use]
extern crate rocket;

use anyhow::{anyhow, Result};
use rocket::http::Status;
use rocket::serde::{json::Json, Deserialize};
use rocket::State as RState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Challenge {
    fqdn: String,
    value: String,
}

struct State {
    challenges: Mutex<HashMap<String, String>>,
}

fn lookup_impl(state: &RState<State>, path: PathBuf) -> Result<Json<Vec<String>>> {
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
        .lock()
        .map_err(|e| anyhow!("expected challenges to lock: {:?}", e))?
        .get(&fqdn)
        .ok_or_else(|| anyhow!("expected challenge under: {}", fqdn))?
        .clone();
    Ok(Json(vec![challenge]))
}

#[get("/lookup/<path..>")]
fn lookup(state: &RState<State>, path: PathBuf) -> Result<Json<Vec<String>>, Status> {
    lookup_impl(state, path).map_err(|e| {
        error!("{:?}", e);
        Status::InternalServerError
    })
}

#[post("/present", data = "<challenge>")]
fn present(state: &RState<State>, challenge: Json<Challenge>) {
    trace!("presented challenge for fqdn: {}", challenge.fqdn);
    state
        .challenges
        .lock()
        .unwrap()
        .insert(challenge.fqdn.clone(), challenge.value.clone());
}

#[post("/cleanup", data = "<challenge>")]
fn cleanup(state: &RState<State>, challenge: Json<Challenge>) {
    state.challenges.lock().unwrap().remove(&challenge.fqdn);
}

#[launch]
fn rocket() -> _ {
    env_logger::init();
    rocket::build()
        .manage(State {
            challenges: Mutex::new(HashMap::new()),
        })
        .mount("/", routes![lookup, present, cleanup])
}
