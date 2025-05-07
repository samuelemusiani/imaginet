use anyhow::{Context, Result};
pub use cable::Cable;
pub use namespace::{NSInterface, Namespace};
use serde::{Deserialize, Serialize};
pub use switch::Switch;

mod cable;
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
        dbg!("check_dependency");
        dbg!(name);

        for con in &self.cables {
            let eqa = con.get_a().split('/').any(|x| x == name);
            let eqb = con.get_b().split('/').any(|x| x == name);
            if eqa || eqb {
                anyhow::bail!("Device {} is connected to a cable", name);
            }
        }

        for ns in &self.namespaces {
            for i in ns.get_interfaces() {
                if i.get_endpoint() == name {
                    anyhow::bail!("Device {} is connected to a namespace", name);
                }
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

pub fn calculate_endpoint_type(t: &Topology, name: &str) -> String {
    for sw in t.get_switches() {
        if sw.get_name() == name {
            return sw.sock_path(".");
        }
    }

    panic!("Endpoint not found");
}
