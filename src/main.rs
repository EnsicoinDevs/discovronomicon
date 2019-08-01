#![feature(proc_macro_hygiene, decl_macro)]

use std::fmt;
#[macro_use]
extern crate rocket;
use rocket::State;
use rocket_contrib::json::Json;
use service_book::{
    Address, PingResponse, RegisterResponse, ServiceIdentity, ServiceList, Session,
};
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

#[post("/discover/<protocol>", data = "<identity>")]
fn register(
    protocol: String,
    identity: Json<Address>,
    trusted: State<LockedBook>,
) -> Result<Json<RegisterResponse>, InternError> {
    Ok(Json(RegisterResponse {
        session: trusted.write()?.add_address(ServiceIdentity {
            protocol,
            address: identity.into_inner().address,
        }),
    }))
}

#[get("/discover/<protocol>")]
fn get(protocol: String, trusted: State<LockedBook>) -> Result<Json<ServiceList>, InternError> {
    Ok(Json(ServiceList {
        trusted: trusted.read()?.get(&protocol).unwrap_or_else(Vec::new),
        untrusted: Vec::new(),
    }))
}

#[put("/ping/<token>")]
fn ping(
    token: rocket_contrib::uuid::Uuid,
    trusted: State<LockedBook>,
) -> Result<Json<PingResponse>, InternError> {
    Ok(Json(PingResponse {
        ack: trusted.write()?.ping(&Session {
            token: token.into_inner(),
        }),
    }))
}

type LockedBook = std::sync::Arc<RwLock<registry::ServiceBook>>;

fn clean_ping(trusted: LockedBook) {
    let duration = std::time::Duration::from_secs(30);
    std::thread::Builder::new()
        .name("cleaner".to_owned())
        .spawn(move || loop {
            std::thread::sleep(duration);
            trusted
                .write()
                .expect("Could not acquire lock")
                .clean(duration);
        })
        .expect("Could not start cleaning");
}

fn main() {
    let book = std::sync::Arc::new(RwLock::new(registry::ServiceBook::new()));
    let rkt = rocket::ignite()
        .manage(book.clone())
        .mount("/", routes![register, get, ping]);
    clean_ping(book);
    rkt.launch();
}
