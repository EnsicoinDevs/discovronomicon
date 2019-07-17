use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct ServiceId {
    pub protocol: String,
    pub address: String,
}

pub struct Register {
    pub id: ServiceId,
}
impl actix::Message for Register {
    type Result = Option<Session>;
}

pub struct Service {
    pub last_seen: std::time::Instant,
    pub id: ServiceId,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Message)]
pub struct Session {
    pub token: Uuid,
}

impl Session {
    pub fn new() -> Session {
        Session {
            token: Uuid::new_v4(),
        }
    }
}

pub struct ServiceBook {
    trusted: HashMap<Session, Service>,
    known: HashSet<ServiceId>,
    //database:
}

impl ServiceBook {
    pub fn new() -> ServiceBook {
        ServiceBook {
            trusted: HashMap::new(),
            known: HashSet::new(),
        }
    }

    pub fn add_address(&mut self, id: ServiceId) -> Option<Session> {
        if self.known.contains(&id) {
            None
        } else {
            let service = Service {
                last_seen: std::time::Instant::now(),
                id: id.clone(),
            };
            self.known.insert(id);
            let connection = Session::new();
            self.trusted.insert(connection, service);
            Some(connection)
        }
    }
}

impl Actor for ServiceBook {
    type Context = Context<Self>;
}

impl Handler<Register> for ServiceBook {
    type Result = Option<Session>;

    fn handle(&mut self, msg: Register, _: &mut Context<Self>) -> Self::Result {
        self.add_address(msg.id)
    }
}
