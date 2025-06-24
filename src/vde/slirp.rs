use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::PID_FILE_NAME;

#[derive(Debug, Serialize, Deserialize)]
pub struct Slirp {
    name: String,
}

impl Slirp {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn get_name(&self) -> &String {
        &self.name
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
        vec![
            "--pidfile".to_owned(),
            self.pid_path(base),
            format!("ptp:///{}/{}", self.base_path(base), self.get_name()),
            "slirp://".to_owned(),
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
