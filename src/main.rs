#![feature(proc_macro_hygiene, decl_macro)]

use std::fmt;
#[macro_use]
extern crate rocket;
use rocket::State;
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;

mod registry;

#[derive(Debug)]
pub enum InternError {
    LockError,
}

impl fmt::Display for InternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternError::LockError => write!(f, "Error in a lock"),
        }
    }
}

impl std::error::Error for InternError {}

impl<T> From<std::sync::PoisonError<T>> for InternError {
    fn from(_: std::sync::PoisonError<T>) -> InternError {
        InternError::LockError
    }
}

#[derive(Serialize, Deserialize)]
struct RegisterRequest {
    pub address: String,
}

#[derive(Serialize, Deserialize)]
struct RegisterResponse {
    pub session: Option<registry::Session>,
}

#[post("/discover/<protocol>", data = "<identity>")]
fn register(
    protocol: String,
    identity: Json<RegisterRequest>,
    trusted: State<LockedBook>,
) -> Result<Json<RegisterResponse>, InternError> {
    Ok(Json(RegisterResponse {
        session: trusted.write()?.add_address(registry::ServiceId {
            protocol,
            address: identity.into_inner().address,
        }),
    }))
}

#[derive(Serialize)]
struct GetResponse {
    pub trusted: Option<HashSet<String>>,
}

#[get("/discover/<protocol>")]
fn get(protocol: String, trusted: State<LockedBook>) -> Result<Json<GetResponse>, InternError> {
    Ok(Json(GetResponse {
        trusted: trusted.read()?.get(&protocol).cloned(),
    }))
}

#[derive(Serialize)]
pub struct PingResponse {
    pub ack: bool,
}

#[put("/ping/<token>")]
fn ping(
    token: rocket_contrib::uuid::Uuid,
    trusted: State<LockedBook>,
) -> Result<Json<PingResponse>, InternError> {
    dbg!(trusted.read().unwrap());
    Ok(Json(PingResponse {
        ack: trusted.write()?.ping(&registry::Session {
            token: token.into_inner(),
        }),
    }))
}

type LockedBook = RwLock<registry::ServiceBook>;

fn main() {
    rocket::ignite()
        .manage(RwLock::new(registry::ServiceBook::new()))
        .mount("/", routes![register, get, ping])
        .launch();
}
