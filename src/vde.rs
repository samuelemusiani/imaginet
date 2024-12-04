pub use switch::Switch;
mod switch;

const PID_FILE_NAME: &str = "pid";
const MGMT_FILE_NAME: &str = "mgmt";
const SOCK_FILE_NAME: &str = "sock";


/// A vde topology is a struct that contains all the necessary 
/// information to create a network topology based on VDE
pub struct Topology {
    switches: Vec<Switch>,
}

impl Topology {
    pub fn new() -> Topology {
        Topology {
            switches: Vec::new(),
        }
    }

    pub fn add_switch(&mut self, sw: Switch) {
        self.switches.push(sw);
    }

    pub fn get_switches(&self) -> &Vec<Switch> {
        &self.switches
    }
}
