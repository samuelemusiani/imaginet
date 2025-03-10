use anyhow::{Context, Result};
pub use connection::Connection;
pub use namespace::{NSInterface, Namespace};
use serde::{Deserialize, Serialize};
pub use switch::Switch;

mod connection;
mod namespace;
mod switch;

const PID_FILE_NAME: &str = "pid";
const MGMT_FILE_NAME: &str = "mgmt";
const SOCK_FILE_NAME: &str = "sock";

/// A vde topology is a struct that contains all the necessary
/// information to create a network topology based on VDE
#[derive(Debug, Serialize, Deserialize)]
pub struct Topology {
    switches: Vec<Switch>,
    namespaces: Vec<Namespace>,
    connections: Vec<Connection>,
}

impl Topology {
    pub fn new() -> Topology {
        Topology {
            switches: Vec::new(),
            namespaces: Vec::new(),
            connections: Vec::new(),
        }
    }

    fn is_name_used(&mut self, name: &str) -> bool {
        for sw in &self.switches {
            if sw.get_name() == name {
                return true;
            }
        }

        for ns in &self.namespaces {
            if ns.get_name() == name {
                return true;
            }
        }

        for con in &self.connections {
            if con.get_name() == name {
                return true;
            }
        }

        false
    }

    pub fn add_switch(&mut self, sw: Switch) -> Result<()> {
        if self.is_name_used(sw.get_name()) {
            anyhow::bail!("Name already used");
        }
        self.switches.push(sw);

        Ok(())
    }

    pub fn add_namespace(&mut self, ns: Namespace) -> Result<()> {
        if self.is_name_used(ns.get_name()) {
            anyhow::bail!("Name already used");
        }
        self.namespaces.push(ns);

        Ok(())
    }

    pub fn add_connection(&mut self, c: Connection) -> Result<()> {
        if self.is_name_used(c.get_name()) {
            anyhow::bail!("Name already used");
        }
        self.connections.push(c);

        Ok(())
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

    pub fn to_string(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(anyhow::Error::new)
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
