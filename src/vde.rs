pub use switch::Switch;
pub use namespace::{Namespace, NSInterface};
pub use connection::Connection;
use serde::{Serialize, Deserialize};
use anyhow::{Context, Result};

mod switch;
mod namespace;
mod connection;

const PID_FILE_NAME: &str = "pid";
const MGMT_FILE_NAME: &str = "mgmt";
const SOCK_FILE_NAME: &str = "sock";


/// A vde topology is a struct that contains all the necessary 
/// information to create a network topology based on VDE
#[derive(Debug, Serialize, Deserialize)]
pub struct Topology {
    switches: Vec<Switch>,
    namespaces: Vec<Namespace>,
    connections: Vec<Connection>
}

impl Topology {
    pub fn new() -> Topology {
        Topology {
            switches: Vec::new(), 
            namespaces: Vec::new(), 
            connections: Vec::new()
        }
    }

    pub fn add_switch(&mut self, sw: Switch) {
        self.switches.push(sw);
    }

    pub fn add_namespace(&mut self, ns: Namespace) {
        self.namespaces.push(ns);
    }

    pub fn add_connection(&mut self, c: Connection) {
        self.connections.push(c);
    }

    pub fn get_switches(&self) -> &Vec<Switch> {
        &self.switches
    }

    pub fn get_namespaces(&self) -> &Vec<Namespace> {
        &self.namespaces
    }

    pub fn get_connections(&self) -> &Vec<Connection> {
        &self.connections
    }

    pub fn to_string(&self) -> String {
        serde_yaml::to_string(self).unwrap()
    }

    pub fn from_string(file: &str) -> Result<Topology> {
        serde_yaml::from_str(&file)
            .map_err(anyhow::Error::new)
            .context("Parsing topology file")
    }
}

pub fn calculate_endpoint_type(t: &Topology, name: &str) -> String {
    for sw in t.get_switches() {
        if sw.get_name() == name {
            return sw.sock_path(".");
        }
    }

    panic!("Endpoint not found");
}
