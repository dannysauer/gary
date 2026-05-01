use super::data::*;
use super::network::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub trait ClusterCommunicator {
    fn send_message(&self, target: &str, msg: &Message) -> bool;
    fn handle_message(&mut self, msg: &Message);
    fn get_nghbr_sample(&self, a: &HashMap<String, DateTime<Utc>>) -> Vec<String>;
    fn comm_recv_gossip(&mut self, payload: &[String]);
    fn update_neighbors(&mut self);
    fn delinquent_node_check(&mut self);
}

pub trait ClusterCommunicationReceiver {
    fn heartbeat_response(peer: &str);
    fn sync_response(info: MachineInfo);
}

pub trait DeploymentCommunicator {
    fn request_run_deployment(deployment: Deployment);
}

pub trait DeploymentCommunicationReceiver {}
