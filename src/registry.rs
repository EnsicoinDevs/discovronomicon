use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ServiceId {
    pub protocol: String,
    pub address: String,
}

#[derive(Debug)]
pub struct Service {
    pub last_seen: std::time::Instant,
    pub id: ServiceId,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
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

#[derive(Debug)]
pub struct ServiceBook {
    trusted: HashMap<Session, Service>,
    known: HashSet<ServiceId>,
    protocol: HashMap<String, HashMap<Session, String>>,
    //database:
}

impl ServiceBook {
    pub fn new() -> ServiceBook {
        ServiceBook {
            trusted: HashMap::new(),
            known: HashSet::new(),
            protocol: HashMap::new(),
        }
    }

    pub fn clean(&mut self, older_than: std::time::Duration) {
        let now = std::time::Instant::now();
        let mut to_remove = Vec::new();
        for (session, service) in &mut self.trusted {
            if now.duration_since(service.last_seen) > older_than {
                match self.protocol.get_mut(&service.id.protocol) {
                    None => (),
                    Some(hash) => {
                        hash.remove(&session);
                    }
                }
                self.known.remove(&service.id);
                to_remove.push(session.clone())
            }
        }
        for k in to_remove {
            self.trusted.remove(&k);
        }
    }

    pub fn ping(&mut self, session: &Session) -> bool {
        match self.trusted.get_mut(session) {
            None => false,
            Some(mut service) => {
                service.last_seen = std::time::Instant::now();
                true
            }
        }
    }

    pub fn get(&self, protocol: &str) -> Option<Vec<String>> {
        self.protocol
            .get(protocol)
            .map(|h| h.values().map(|a| a.clone()).collect())
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
            self.protocol
                .entry(service.id.protocol.clone())
                .or_insert_with(HashMap::new)
                .insert(connection.clone(), service.id.address.clone());
            self.trusted.insert(connection, service);
            Some(connection)
        }
    }
}
