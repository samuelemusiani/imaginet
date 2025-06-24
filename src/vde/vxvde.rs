use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::PID_FILE_NAME;

#[derive(Debug, Serialize, Deserialize)]
pub struct VXVDE {
    name: String,
    addr: Option<String>,
    port: Option<u16>,
}

impl VXVDE {
    pub fn new(name: String) -> Self {
        Self {
            name,
            addr: None,
            port: None,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_addr(&mut self, addr: String) {
        self.addr = Some(addr);
    }

    pub fn get_addr(&self) -> Option<&String> {
        self.addr.as_ref()
    }

    pub fn set_port(&mut self, port: u16) {
        self.port = Some(port);
    }

    pub fn get_port(&self) -> Option<u16> {
        self.port
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
        "vde_plug".to_owned()
    }

    /// base: base path for the working directory.
    pub fn exec_args(&self, base: &str) -> Vec<String> {
        let mut vxconn = "vxvde://".to_owned();

        if self.addr.is_some() {
            vxconn = vxconn + self.addr.as_ref().unwrap();
        }

        if self.port.is_some() {
            vxconn = vxconn + "/port=" + &self.port.unwrap().to_string();
        }

        vec![
            "--pidfile".to_owned(),
            self.pid_path(base),
            format!("ptp:///{}/{}", self.base_path(base), self.get_name()),
            vxconn,
        ]
    }

    /// Get the path of the interface connection given the global base path
    pub fn conn_path(&self, base: &str) -> Result<String> {
        Ok(PathBuf::from(self.base_path(base))
            .join(self.get_name())
            .to_str()
            .unwrap()
            .to_owned())
    }
}
