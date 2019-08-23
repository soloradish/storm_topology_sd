use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::storm_client::TopologyWorkers;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct FileSD {
    targets: BTreeSet<String>,
    labels: BTreeMap<String, String>,
}

impl From<TopologyWorkers> for FileSD {
    fn from(tw: TopologyWorkers) -> Self {
        let mut targets = BTreeSet::new();
        for hp in tw.host_port_list {
            let target = format!("{}:{}", &hp.host, &hp.port);
            targets.insert(String::from(target));
        }
        let mut labels = BTreeMap::new();
        labels.insert("topology_id".to_string(), tw.id.clone());

        FileSD { targets, labels }
    }
}

impl FileSD {
    pub fn add_label(&mut self, k: &str, v: &str) {
        self.labels.insert(k.to_string(), v.to_string());
    }
}
