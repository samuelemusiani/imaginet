pub use switch::Switch;
pub use namespace::{Namespace, NSInterface};
mod switch;
mod namespace;

const PID_FILE_NAME: &str = "pid";
const MGMT_FILE_NAME: &str = "mgmt";
const SOCK_FILE_NAME: &str = "sock";


/// A vde topology is a struct that contains all the necessary 
/// information to create a network topology based on VDE
pub struct Topology {
    switches: Vec<Switch>,
    namespaces: Vec<Namespace>
}

impl Topology {
    pub fn new() -> Topology {
        Topology {
            switches: Vec::new(), namespaces: Vec::new()
        }
    }

    pub fn add_switch(&mut self, sw: Switch) {
        self.switches.push(sw);
    }

    pub fn add_namespace(&mut self, ns: Namespace) {
        self.namespaces.push(ns);
    }

    pub fn get_switches(&self) -> &Vec<Switch> {
        &self.switches
    }

    pub fn get_namespaces(&self) -> &Vec<Namespace> {
        &self.namespaces
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
