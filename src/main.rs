use actix::prelude::*;
use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use std::fmt;

mod registry;

#[derive(Debug)]
pub enum InternError {
    LockError,
    MailError(MailboxError),
}

impl fmt::Display for InternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternError::LockError => write!(f, "Error in a lock"),
            InternError::MailError(e) => write!(f, "Error in intern sending message: {}", e),
        }
    }
}

impl std::error::Error for InternError {}

impl<T> From<std::sync::PoisonError<T>> for InternError {
    fn from(err: std::sync::PoisonError<T>) -> InternError {
        InternError::LockError
    }
}

impl From<MailboxError> for InternError {
    fn from(err: MailboxError) -> InternError {
        InternError::MailError(err)
    }
}

fn register(
    data: web::Data<Addr<registry::ServiceBook>>,
    path: web::Path<registry::ServiceId>,
) -> impl Future<Item = Option<registry::Session>, Error = InternError> {
    data.recipient()
        .send(registry::Register {
            id: path.into_inner(),
        })
        .map_err(InternError::MailError)
}

fn main() {
    let reg = registry::ServiceBook::new();
    let reg_addr = reg.start();
    HttpServer::new(|| {
        App::new().data(reg_addr).route(
            "/register/{protocol}/{address}",
            web::get().to_async(register),
        )
    })
    .bind("0.0.0.0:3333")
    .unwrap()
    .run()
    .unwrap();
}
