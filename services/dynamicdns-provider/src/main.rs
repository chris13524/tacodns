#[macro_use]
extern crate rocket;

use rocket::response::status::NotFound;
use rocket::serde::json::Json;
use rocket::State;
use std::sync::Mutex;

struct DynamicDns {
    record: Mutex<Option<String>>,
}

#[get("/get")]
fn get(dynamic_dns: &State<DynamicDns>) -> Result<Json<Vec<String>>, NotFound<String>> {
    match *dynamic_dns.record.lock().unwrap() {
        Some(ref record) => Ok(Json(vec![record.clone()])),
        None => Err(NotFound("Could not find record".to_string())),
    }
}

#[post("/set", data = "<record>")]
fn set(dynamic_dns: &State<DynamicDns>, record: String) {
    *dynamic_dns.record.lock().unwrap() = Some(record);
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(DynamicDns {
            record: Mutex::new(None),
        })
        .mount("/", routes![get, set])
}
