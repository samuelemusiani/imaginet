use std::path::PathBuf;
use super::{MGMT_FILE_NAME, PID_FILE_NAME, SOCK_FILE_NAME};
use serde::{Serialize, Deserialize};

const DEFAULT_PORTS: u32 = 10;

/// This is the internal rappresentation of a switch

#[derive(Debug, Serialize, Deserialize)]
pub struct Switch {
    /// The name should be unique
    name: String,
    config: Vec<String>,
    ports: u32,
}

impl Switch {
    pub fn new(name: String) -> Switch {
        Switch { name, config: Vec::new(), ports: DEFAULT_PORTS }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn add_config(&mut self, config: String) {
        self.config.push(config);
    }

    pub fn set_ports(&mut self, ports: u32) {
        self.ports = ports;
    }

    pub fn get_config(&self) -> &Vec<String> {
        &self.config
    }

    pub fn needs_config(&self) -> bool {
        !self.config.is_empty()
    }

    /// Get base path of all the files related to the switch given
    /// the global base path
    pub fn base_path(&self, base: &str) -> String {
        PathBuf::from(base).join(&self.name).to_str().unwrap().to_owned()
    }

    /// Get the path of the pid file of the switch given the global base path
    pub fn pid_path(&self, base: &str) -> String {
        PathBuf::from(self.base_path(base)).join(PID_FILE_NAME).to_str().unwrap().to_owned()
    }

    /// Get the path of the management file of the switch given the global base path
    pub fn mgmt_path(&self, base: &str) -> String {
        PathBuf::from(self.base_path(base)).join(MGMT_FILE_NAME).to_str().unwrap().to_owned()
    }

    /// Get the path of the socket file of the switch given the global base path
    pub fn sock_path(&self, base: &str) -> String {
        PathBuf::from(self.base_path(base)).join(SOCK_FILE_NAME).to_str().unwrap().to_owned()
    }

    pub fn config_path(&self, base: &str) -> String {
        PathBuf::from(self.base_path(base)).join("config").to_str().unwrap().to_owned()
    }

    pub fn exec_command(&self) -> String {
        String::from("vde_switch")
    }

    pub fn exec_args(&self, base: &str) -> Vec<String> {
        let pid_p = self.pid_path(base);
        let mgmt_p = self.mgmt_path(base);
        let sock_p = self.sock_path(base);
        let conf_p = self.config_path(base);

        vec!["--pidfile".to_owned(), pid_p, 
            "--mgmt".to_owned(), mgmt_p, 
            "--sock".to_owned(), sock_p,
            "--rcfile".to_owned(), conf_p,
            "--numports".to_owned(), self.ports.to_string(),
            "--daemon".to_owned()]
    }

    pub fn attach_command(&self) -> String {
        String::from("vdeterm")
    }

    pub fn attach_args(&self, base: &str, _pid: u32) -> Vec<String> {
        let sock_p = self.mgmt_path(base);
        vec![sock_p]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_switch() {
        let name = "test";
        let sw = Switch::new(name.to_owned());

        assert_eq!(sw.get_name(), name);
    }

    #[test]
    fn switch_base_path() {
        let name = "lara";
        let sw = Switch::new(name.to_owned());
        let base = "/tmp";

        assert_eq!(sw.base_path(base), String::from("/tmp/lara"));
    }

    #[test]
    fn switch_pid_path() {
        let name = "maasldkf";
        let sw = Switch::new(name.to_owned());
        let base = "/tmp";

        assert_eq!(sw.pid_path(base), String::from("/tmp/maasldkf/pid"));
    }

    #[test]
    fn switch_mgmt_path() {
        let name = "sdfk3i";
        let sw = Switch::new(name.to_owned());
        let base = "/tmp";

        assert_eq!(sw.mgmt_path(base), String::from("/tmp/sdfk3i/mgmt"));
    }

    #[test]
    fn switch_sock_path() {
        let name = "sw-13ndo28";
        let sw = Switch::new(name.to_owned());
        let base = "/tmp";

        assert_eq!(sw.sock_path(base), String::from("/tmp/sw-13ndo28/sock"));
    }
}
