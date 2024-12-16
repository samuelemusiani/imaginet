use serde::{Serialize, Deserialize};
use anyhow::{Context, Result};
use std::collections::{HashSet, HashMap};
use std::net;

const DEFAULT_SWITCH_PORTS: u32 = 32;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub name: String,
    pub port: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Switch {
    pub name: String,
    pub config: Option<String>,
    pub ports: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    pub name: String,
    pub interfaces: Vec<NSInterface>,
    commands: Option<Vec<String>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NSInterface {
    pub name: String,
    pub ip: String,
    pub endpoint: Endpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub name: String,
    pub endpoint_a: Endpoint,
    pub endpoint_b: Endpoint,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub switch: Option<Vec<Switch>>,
    pub namespace: Option<Vec<Namespace>>,
    pub connections: Option<Vec<Connection>>
}

impl Config {
    pub fn from_string(file: &str) -> Result<Config> {
        let c = serde_yaml::from_str::<Self>(&file)
            .context("Deserialize config file failed")?;

        c.checks().context("Config checks failed")?;

        Ok(c)
    }

    fn checks(&self) -> Result<()>{
        // All names must be unique

        let mut set = HashSet::new();

        if let Some(ns) = &self.namespace {
            for n in ns {
                if !set.insert(&n.name) {
                    anyhow::bail!("Namespace name {} is not unique", n.name);
                }
            }
        }

        if let Some(sw) = &self.switch {
            for s in sw {
                if !set.insert(&s.name) {
                    anyhow::bail!("Switch name {} is not unique", s.name);
                }

                if let Some(port) = s.ports {
                    if port == 0 {
                        anyhow::bail!("Switch {} has 0 ports", s.name);
                    }
                }
            }
        }

        if let Some(con) = &self.connections {
            for c in con {
                if !set.insert(&c.name) {
                    anyhow::bail!("Connection name {} is not unique", c.name);
                }
            }
        }

        drop(set);

        // Endpoints must exist and ports must be valid

        let mut map = HashMap::new();

        if let Some(sw) = &self.switch {
            for s in sw {
                // To check if the port endpoint is valid we reuse the Endpoint struct,
                // but with a different purpose for the port field. In this case, the port
                // field is used to store the number of ports of the switch.
                let ports = match s.ports {
                    Some(p) => p,
                    None => DEFAULT_SWITCH_PORTS,
                };
                map.insert(&s.name, Endpoint { name: s.name.clone(), port: Some(ports) });
            }
        }

        // To avoid another function we use the endpoint_check closure.
        // This simply checks if the endpoint exists and if the port is valid.
        // based on the map we created before.
        let endpoint_check = |name: String, port: Option<u32>| -> Result<()> {
            let end = map.get(&name)
                .ok_or_else(|| anyhow::anyhow!("Endpoint {name} does not exist"))?;

            if let Some(p) = port {
                let end_ports = end.port.expect("Internal error: port field is None");
                if p >= end_ports {
                    let mut s = String::new();
                    if p == end_ports {
                        s.push_str("\nPorts are zero-indexed, so you should decrement the port number by one :)");
                    }
                    anyhow::bail!("Port {p} is out of range for endpoint {name} (max {end_ports} ports){s}");
                };
            };

            Ok(())
        };

        if let Some(ns) = &self.namespace {
            for n in ns {
                for i in &n.interfaces {
                    endpoint_check(i.endpoint.name.clone(), i.endpoint.port)
                        .context(format!("Checks failed for interface {} on namespace {}", i.name, n.name))?;
                }
            }
        }

        if let Some(con) = &self.connections {
            for c in con {
                endpoint_check(c.endpoint_a.name.clone(), c.endpoint_a.port)
                    .context(format!("Checks failed for connection {} endpoint A", c.name))?;
                endpoint_check(c.endpoint_b.name.clone(), c.endpoint_b.port)
                    .context(format!("Checks failed for connection {} endpoint B", c.name))?;
            }
        }

        // Specific checks
        if let Some(ns) = &self.namespace {
            for n in ns {
                n.checks().context(format!("Checks failed for namespace {}", n.name))?;
            }
        }

        Ok(())
    }
}

impl Namespace {
    fn checks(&self) -> Result<()> {

        for i in &self.interfaces {
            i.checks().context(format!("Checks failed for interface {}", i.name))?;
        }

        Ok(())
    }
}

impl NSInterface {
    fn checks(&self) -> Result<()> {
        // Check if IP is valid in CIDR notation
        let (ip, mask) = match self.ip.find('/') {
            Some(p) => (&self.ip[..p], &self.ip[p+1..]),
            None => anyhow::bail!("Invalid CIDR format, missing /"),
        };
        let res = ip.parse::<net::IpAddr>()
            .context(format!("IP address: {}", self.ip))?;

        let m = mask.parse::<u8>().context("Invalid mask, not a number")?;
        match res {
            net::IpAddr::V4(_) => {
                if m > 32 {
                    anyhow::bail!("Invalid mask, too large for IPv4 (> 32)");
                }
            },
            net::IpAddr::V6(_) => {
                if m > 128 {
                    anyhow::bail!("Invalid mask, too large for IPv6 (> 128)");
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_config() {
        let file = r#"
switch:
    - name: "test"
      config: "test.conf"
"#;
        let c = Config::from_string(file).unwrap();
        let sws = c.switch.unwrap();
        assert_eq!(sws.len(), 1);
        let sw = &sws[0];
        assert_eq!(sw.name, "test");
        assert_eq!(sw.config, Some("test.conf".to_owned()));
    }
}
