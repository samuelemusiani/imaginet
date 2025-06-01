use anyhow::{Context, Ok, Result};
pub use cable::Cable;
use core::fmt;
pub use namespace::{NSInterface, Namespace};
use serde::{Deserialize, Serialize};
pub use switch::Switch;

mod cable;
mod namespace;
mod switch;

const PID_FILE_NAME: &str = "pid";
const CONF_FILE_NAME: &str = "config";
const MGMT_FILE_NAME: &str = "mgmt";
const SOCK_FILE_NAME: &str = "sock";
pub const OPEN_DIR_NAME: &str = "opn";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum VdeConnProtocols {
    VDE,
    PTP,
}

impl fmt::Display for VdeConnProtocols {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// A vde topology is a struct that contains all the necessary
/// information to create a network topology based on VDE
#[derive(Debug, Serialize, Deserialize)]
pub struct Topology {
    switches: Vec<Switch>,
    namespaces: Vec<Namespace>,
    cables: Vec<Cable>,
}

impl Topology {
    pub fn new() -> Topology {
        Topology {
            switches: Vec::new(),
            namespaces: Vec::new(),
            cables: Vec::new(),
        }
    }

    pub fn is_name_used(&self, name: &str) -> bool {
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

        for con in &self.cables {
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

    pub fn add_cable(&mut self, c: Cable) -> Result<()> {
        if self.is_name_used(c.get_name()) {
            anyhow::bail!("Name already used");
        }
        self.cables.push(c);

        Ok(())
    }

    pub fn get_switches(&self) -> &Vec<Switch> {
        &self.switches
    }

    pub fn get_namespaces(&self) -> &Vec<Namespace> {
        &self.namespaces
    }

    pub fn get_cables(&self) -> &Vec<Cable> {
        &self.cables
    }

    pub fn remove_device(&mut self, name: &String) -> Result<()> {
        self.check_dependecy(name)?;

        if let Some(pos) = self.switches.iter().position(|x| x.get_name() == name) {
            self.switches.remove(pos);
            return Ok(());
        };

        if let Some(pos) = self.namespaces.iter().position(|x| x.get_name() == name) {
            self.namespaces.remove(pos);
            return Ok(());
        };

        if let Some(pos) = self.cables.iter().position(|x| x.get_name() == name) {
            self.cables.remove(pos);
            return Ok(());
        };

        Ok(())
    }

    fn check_dependecy(&self, name: &String) -> Result<()> {
        for con in &self.cables {
            let eqa = con.get_a().get_name().split('/').any(|x| x == name);
            let eqb = con.get_b().get_name().split('/').any(|x| x == name);
            if eqa || eqb {
                anyhow::bail!("Device {} is connected to a cable", name);
            }
        }

        Ok(())
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

pub fn find_endpoint_path(
    t: &Topology,
    name: &str,
    port: Option<&String>,
    open: Option<bool>,
) -> Result<String> {
    if open == Some(true) {
        return Ok(format!("{OPEN_DIR_NAME}/{name}"));
    }

    for sw in t.get_switches() {
        if sw.get_name() == name {
            return Ok(sw.sock_path("."));
        }
    }

    // For the namespaces the port must be defined
    let port =
        port.ok_or_else(|| anyhow::anyhow!("Port is not defined and namespaces requires it"))?;

    for ns in t.get_namespaces() {
        if ns.get_name() == name {
            return ns.conn_path(".", port);
        }
    }

    panic!("Endpoint not found");
}

pub fn find_endpoint_protocol(t: &Topology, name: &str) -> Result<VdeConnProtocols> {
    for sw in t.get_switches() {
        if sw.get_name() == name {
            return Ok(VdeConnProtocols::VDE);
        }
    }

    for ns in t.get_namespaces() {
        if ns.get_name() == name {
            return Ok(VdeConnProtocols::PTP);
        }
    }

    panic!("Endpoint not found");
}
