use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait IdChecksum {
    fn id_checksum(&self) -> (String, u64);
}

#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub update_timestamp: u64,
    pub state: HashMap<String, u64>,
}

pub struct Diff<T> {
    pub added: Vec<T>,
    pub changed: Vec<T>,
}

impl Snapshot {
    pub fn new<T: IdChecksum>(items: Vec<T>) -> Self {
        let mut map = HashMap::new();
        for item in items {
            let (id, checksum) = item.id_checksum();
            map.insert(id, checksum);
        }
        Snapshot {
            update_timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time should be after UNIX EPOCH")
                .as_secs(),
            state: map,
        }
    }

    pub fn diff<T: IdChecksum>(&self, items: Vec<T>) -> Diff<T> {
        let mut changed: Vec<T> = Vec::new();
        let mut added: Vec<T> = Vec::new();
        for item in items {
            let (id, checksum) = item.id_checksum();
            match self.state.get(&id) {
                Some(oldchecksum) if *oldchecksum != checksum => changed.push(item),
                None => added.push(item),
                _ => { /* do nothing */ }
            }
        }

        Diff {
            changed: changed,
            added: added,
        }
    }
}
