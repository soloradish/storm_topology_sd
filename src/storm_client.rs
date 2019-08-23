use crate::error::{self, Result};
use reqwest;
use snafu::ResultExt;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct StormRestClient {
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct TopologySummary {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "id")]
    pub id: String,
    // todo: use enum map status
    #[serde(rename = "status")]
    pub status: String,
    #[serde(rename = "uptimeSeconds")]
    pub uptime_seconds: i64,

    #[serde(rename = "tasksTotal")]
    pub task_total: i64,
    #[serde(rename = "workersTotal")]
    pub worker_total: i64,
    #[serde(rename = "executorsTotal")]
    pub executor_total: i64,
    #[serde(rename = "replicationCount")]
    pub replication_total: i64,
}

#[derive(Debug, Deserialize)]
pub struct TopologySummaries {
    #[serde(rename = "topologies")]
    pub topologies: Vec<TopologySummary>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct HostPort {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct TopologyWorkers {
    #[serde(default = "default_id")]
    pub id: String,
    #[serde(rename = "hostPortList")]
    pub host_port_list: Vec<HostPort>,
    #[serde(rename = "logviewerPort")]
    pub logviewer_port: u16,
}

fn default_id() -> String {
    "".to_string()
}

impl StormRestClient {
    pub fn new(storm_ui_server: &str) -> Self {
        debug!("Init new storm client, url: {}", storm_ui_server);
        Self {
            url: storm_ui_server.trim_end_matches("/").to_string(),
        }
    }

    fn topology_summary_endpoint(&self) -> String {
        format!("{}/api/v1/topology/summary", self.url)
    }

    fn topology_workers_endpoint(&self, topology_id: &str) -> String {
        format!("{}/api/v1/topology-workers/{}", self.url, topology_id)
    }

    pub fn list_active_topologies(&self) -> Result<Vec<String>> {
        let url = self.topology_summary_endpoint();

        debug!("Start list topologies...");
        let topologies: TopologySummaries = reqwest::get(&url)
            .context(error::HttpError {
                url: url.to_string(),
            })?
            .json()
            .context(error::ParseError)?;

        debug!("Succeed list topologies, [{:?}]", topologies);
        Ok(topologies
            .topologies
            .iter()
            .filter(|x| x.status == "ACTIVE")
            .map(|x| x.id.to_string())
            .collect())
    }

    pub fn get_topology_workers(&self, topology_id: &str) -> Result<TopologyWorkers> {
        let url = self.topology_workers_endpoint(topology_id);

        debug!("start get topology workers, id: {}", topology_id);
        let mut workers: TopologyWorkers = reqwest::get(&url)
            .context(error::HttpError {
                url: url.to_string(),
            })?
            .json()
            .context(error::ParseError)?;
        workers.id = topology_id.to_string();

        debug!("Succeed get topology workers, [{:?}]", workers);
        Ok(workers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_summary_serde() {
        let s = "{
            \"topologies\": [{
                \"id\": \"WordCount3-1-1402960825\",
                \"name\": \"WordCount3\",
                \"status\": \"ACTIVE\",
                \"uptime\": \"6m 5s\",
                \"uptimeSeconds\": 365,
                \"tasksTotal\": 28,
                \"workersTotal\": 3,
                \"executorsTotal\": 28,
                \"replicationCount\": 1,
                \"requestedMemOnHeap\": 640,
                \"requestedMemOffHeap\": 128,
                \"requestedTotalMem\": 768,
                \"requestedCpu\": 80,
                \"assignedMemOnHeap\": 640,
                \"assignedMemOffHeap\": 128,
                \"assignedTotalMem\": 768,
                \"assignedCpu\": 80
            }],
            \"schedulerDisplayResource\": true
        }";

        let topologies: TopologySummaries = serde_json::from_str(s).unwrap();
        assert_eq!(topologies.topologies.len(), 1);

        let topology = &topologies.topologies[0];
        assert_eq!(topology.name, "WordCount3");
        assert_eq!(topology.id, "WordCount3-1-1402960825");
        assert_eq!(topology.status, "ACTIVE");
        assert_eq!(topology.uptime_seconds, 365);
        assert_eq!(topology.task_total, 28);
        assert_eq!(topology.worker_total, 3);
        assert_eq!(topology.executor_total, 28);
        assert_eq!(topology.replication_total, 1);
    }

    #[test]
    fn test_host_port_list_serde() {
        let s = "{
            \"hostPortList\": [\
                {\"host\":\"192.168.202.2\", \"port\":6701},\
                {\"host\":\"192.168.202.2\", \"port\":6702},\
                {\"host\":\"192.168.202.3\", \"port\":6700}],\
            \"logviewerPort\": 8000}";
        let workers: TopologyWorkers = serde_json::from_str(s).unwrap();

        assert_eq!(workers.host_port_list[0].host, "192.168.202.2");
        assert_eq!(workers.host_port_list[0].port, 6701u16);
        assert_eq!(workers.logviewer_port, 8000u16);
    }
}
