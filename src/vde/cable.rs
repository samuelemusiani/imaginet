use super::{VdeConnProtocols, MGMT_FILE_NAME, PID_FILE_NAME};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cable {
    pub name: String,
    pub a: Endpoint,
    pub b: Endpoint,
    pub wirefilter: bool,
    pub config: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Endpoint {
    name: String,
    port: Option<String>,
    protocol: VdeConnProtocols,
}

impl Endpoint {
    pub fn new(name: String, port: Option<String>, protocol: VdeConnProtocols) -> Self {
        Self {
            name,
            port,
            protocol,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_port(&self) -> Option<&String> {
        self.port.as_ref()
    }

    pub fn get_protocol(&self) -> &VdeConnProtocols {
        &self.protocol
    }
}

impl Cable {
    pub fn new(
        name: String,
        a: String,
        port_a: Option<String>,
        protocol_a: VdeConnProtocols,
        b: String,
        port_b: Option<String>,
        protocol_b: VdeConnProtocols,
        wirefilter: Option<bool>,
    ) -> Self {
        Self {
            name,
            a: Endpoint::new(a, port_a, protocol_a),
            b: Endpoint::new(b, port_b, protocol_b),
            wirefilter: wirefilter.unwrap_or(false),
            config: Vec::new(),
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn has_wirefilter(&self) -> bool {
        self.wirefilter
    }

    pub fn add_config(&mut self, config: String) {
        self.config.push(config);
    }

    pub fn get_config(&self) -> &Vec<String> {
        &self.config
    }

    pub fn get_a(&self) -> &Endpoint {
        &self.a
    }

    pub fn get_b(&self) -> &Endpoint {
        &self.b
    }

    pub fn needs_config(&self) -> bool {
        !self.config.is_empty()
    }

    pub fn base_path(&self, base: &str) -> String {
        PathBuf::from(base)
            .join(&self.name)
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub fn pid_path(&self, base: &str) -> String {
        PathBuf::from(self.base_path(base))
            .join(PID_FILE_NAME)
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub fn mgmt_path(&self, base: &str) -> Result<String> {
        if !self.wirefilter {
            return Err(anyhow::anyhow!(
                "No wirefilter cable. Can't have a management file"
            ));
        }
        Ok(PathBuf::from(self.base_path(base))
            .join(MGMT_FILE_NAME)
            .to_str()
            .unwrap()
            .to_owned())
    }

    pub fn config_path(&self, base: &str) -> String {
        PathBuf::from(self.base_path(base))
            .join("config")
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub fn exec_command(&self) -> String {
        if self.wirefilter {
            String::from("dpipe")
        } else {
            String::from("vde_plug")
        }
    }

    pub fn exec_args(&self, base: &str) -> Vec<String> {
        let b = PathBuf::from(base);
        let mut pa = b.join(&self.a.get_name()).to_str().unwrap().to_owned();
        let mut pb = b.join(&self.b.get_name()).to_str().unwrap().to_owned();

        pa = match *self.a.get_protocol() {
            VdeConnProtocols::VDE => format!("vde://{pa}"),
            VdeConnProtocols::PTP => format!("ptp://{pa}"),
        };

        pb = match *self.b.get_protocol() {
            VdeConnProtocols::VDE => format!("vde://{pb}"),
            VdeConnProtocols::PTP => format!("ptp://{pb}"),
        };

        if let Some(port) = &self.a.get_port() {
            if *self.a.get_protocol() == VdeConnProtocols::VDE {
                pa.push_str(&format!("[{port}]"));
            }
        }

        if let Some(port) = &self.b.get_port() {
            if *self.b.get_protocol() == VdeConnProtocols::VDE {
                pb.push_str(&format!("[{port}]"));
            }
        }

        let pid_p = self.pid_path(base);

        if self.wirefilter {
            let mgmt_p = self.mgmt_path(base).unwrap();
            let conf_p = self.config_path(base);

            vec![
                "--daemon".to_owned(),
                "--pidfile".to_owned(),
                pid_p,
                "vde_plug".to_owned(),
                pa,
                "=".to_owned(),
                "wirefilter".to_owned(),
                "--mgmt".to_owned(),
                mgmt_p,
                "--rcfile".to_owned(),
                conf_p,
                "=".to_owned(),
                "vde_plug".to_owned(),
                pb,
            ]
        } else {
            vec![
                pa,
                pb,
                "--pidfile".to_owned(),
                self.pid_path(base),
                "--descr".to_owned(),
                self.name.to_owned(),
                "--daemon".to_owned(),
            ]
        }
    }

    pub fn attach_command(&self) -> Result<String> {
        if self.wirefilter {
            Ok(String::from("vdeterm"))
        } else {
            Err(anyhow::anyhow!(
                "Simple cable (no wirefilter) can't be attached"
            ))
        }
    }

    pub fn attach_args(&self, base: &str) -> Result<Vec<String>> {
        if self.wirefilter {
            let socke_p = self.mgmt_path(base)?;
            Ok(vec![socke_p])
        } else {
            Err(anyhow::anyhow!(
                "Simple cable (no wirefilter) can't be attached"
            ))
        }
    }

    /// Returns the command to execute in order to execute a command
    /// inside the switch. This is different from exec_command in which the
    /// command returned is used to start the switch
    pub fn exec_command_command(&self) -> Result<String> {
        if self.wirefilter {
            Ok(String::from("vdecmd"))
        } else {
            Err(anyhow::anyhow!(
                "Simple cable (no wirefilter) can't be attached"
            ))
        }
    }

    /// Returns the arguments to execute in order to execute a command inside
    /// the switch. This is different from exec_args in which the arguments
    /// returned are used to start the switch. This function is used with
    /// the exec_command_command function
    pub fn exec_command_args(&self, base: &str, command: &mut Vec<String>) -> Result<Vec<String>> {
        if !self.wirefilter {
            return Err(anyhow::anyhow!(
                "Simple cable (no wirefilter) can't be attached"
            ));
        }

        let mut args = vec!["-s".to_owned(), self.mgmt_path(base)?];
        args.append(command);

        return Ok(args);
    }
}
