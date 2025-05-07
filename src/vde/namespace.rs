use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::PID_FILE_NAME;

const STARTER_SCRIPT: &[u8] = include_bytes!("ns_starter.sh");

#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    name: String,
    interfaces: Vec<NSInterface>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NSInterface {
    name: String,
    ip: String,
}

impl Namespace {
    pub fn new(name: String) -> Namespace {
        Namespace {
            name,
            interfaces: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_interfaces(&self) -> &Vec<NSInterface> {
        &self.interfaces
    }

    pub fn add_interface(&mut self, interface: NSInterface) {
        self.interfaces.push(interface);
    }

    /// Get base path of all the files related to the switch given
    /// the global base path
    pub fn base_path(&self, base: &str) -> String {
        PathBuf::from(base)
            .join(&self.name)
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub fn pid_path(&self, base: &str) -> String {
        // Path is written by the ns_starter.sh script
        PathBuf::from(self.base_path(base))
            .join(PID_FILE_NAME)
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub fn exec_command(&self) -> String {
        "vdens".to_owned()
    }

    /// Get the path of the interface connection given the global base path and
    /// the interface name
    pub fn conn_path(&self, base: &str, interface: &str) -> Result<String> {
        for i in &self.interfaces {
            if i.name != interface {
                continue;
            }

            return Ok(PathBuf::from(self.base_path(base))
                .join(&i.name)
                .to_str()
                .unwrap()
                .to_owned());
        }

        anyhow::bail!(
            "No interface found on {} that match the name {}",
            &self.name,
            interface
        );
    }

    /// base: base path for the working directory.
    /// starter: the name of the starter script that will perform pid writing
    pub fn exec_args(&self, base: &str, starter: &str) -> Vec<String> {
        if self.interfaces.len() == 0 {
            return vec![];
        }

        let name = self.get_name().to_owned();
        let mut args = vec!["--hostname".to_owned(), name.clone(), "--multi".to_owned()];

        let b = PathBuf::from(self.base_path(base));
        for i in &self.interfaces {
            let p = b.join(&i.name).to_str().unwrap().to_owned();
            args.push(format!("ptp://{p}"));
        }

        let mut args2 = vec!["--".to_owned(), starter.to_owned(), self.pid_path(base)];
        args.append(&mut args2);
        return args;
    }

    pub fn attach_command(&self) -> String {
        "nsenter".to_owned()
    }

    pub fn attach_args(&self, _base: &str, pid: u32) -> Vec<String> {
        vec![
            "-t".to_owned(),
            pid.to_string(),
            "--preserve-credentials".to_owned(),
            "-U".to_owned(),
            "-u".to_owned(),
            "-n".to_owned(),
            "--keep-caps".to_owned(),
        ]
    }

    /// Returns the command to execute in order to execute a command
    /// inside the namespace. This is different from exec_command in which the
    /// command returned is used to start the namespace
    pub fn exec_command_command(&self) -> String {
        self.attach_command()
    }

    /// Returns the arguments to execute in order to execute a command inside
    /// the namespace. This is different from exec_args in which the arguments
    /// returned are used to start the namespace. This function is used with
    /// the exec_command_command function
    pub fn exec_command_args(
        &self,
        _base: &str,
        pid: u32,
        command: &mut Vec<String>,
    ) -> Vec<String> {
        let mut args = self.attach_args(_base, pid);
        args.append(command);

        return args;
    }

    pub fn get_starter_script() -> &'static [u8] {
        STARTER_SCRIPT
    }
}

impl NSInterface {
    pub fn new(name: String, ip: String) -> NSInterface {
        NSInterface { name, ip }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_ip(&self) -> &String {
        &self.ip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_namespace() {
        let name = "test";
        let ns = Namespace::new(name.to_owned());

        assert_eq!(ns.get_name(), name);
    }
}
