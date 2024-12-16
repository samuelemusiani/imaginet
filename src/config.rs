use serde::{Serialize, Deserialize};
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::net;

#[derive(Debug, Serialize, Deserialize)]
pub struct Switch {
    pub name: String,
    pub config: Option<String>
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
    pub endpoint: String,
    pub port: Option<u32>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub name: String,
    pub a: String,
    pub port_a: Option<u32>,
    pub b: String,
    pub port_b: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub switch: Option<Vec<Switch>>,
    pub namespace: Option<Vec<Namespace>>,
    pub connections: Option<Vec<Connection>>
}

impl Config {
    pub fn from_string(file: &str) -> Result<Config> {
        let c = serde_yaml::from_str(&file)
            .context("Deserialize config file failed")?;

        checks(&c).context("Config checks failed")?;

        Ok(c)
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
        // Check if IP is valid
        let (ip, mask) = match self.ip.find('/') {
            Some(p) => (&self.ip[..p], &self.ip[p+1..]),
            None => bail!("Invalid CIDR format, missing /"),
        };
        let res = ip.parse::<net::IpAddr>()
            .context(format!("IP address: {}", self.ip))?;

        let m = mask.parse::<u8>().context("Invalid mask, not a number")?;
        match res {
            net::IpAddr::V4(_) => {
                if m > 32 {
                    bail!("Invalid mask, too large for IPv4 (> 32)");
                }
            },
            net::IpAddr::V6(_) => {
                if m > 128 {
                    bail!("Invalid mask, too large for IPv6 (> 128)");
                }
            }
        };

        Ok(())
    }
}

fn checks(c: &Config) -> Result<()>{
    // All names must be unique

    let mut set = HashSet::new();

    if let Some(ns) = &c.namespace {
        for n in ns {
            if !set.insert(&n.name) {
                anyhow::bail!("Namespace name {} is not unique", n.name);
            }
        }
    }

    if let Some(sw) = &c.switch {
        for s in sw {
            if !set.insert(&s.name) {
                anyhow::bail!("Switch name {} is not unique", s.name);
            }
        }
    }
    
    if let Some(con) = &c.connections {
        for c in con {
            if !set.insert(&c.name) {
                anyhow::bail!("Connection name {} is not unique", c.name);
            }
        }
    }

    drop(set);

    // Endpoints must exist

    let mut set = HashSet::new();

    if let Some(sw) = &c.switch {
        for s in sw {
            set.insert(&s.name);
        }
    }

    if let Some(ns) = &c.namespace {
        for n in ns {
            for i in &n.interfaces {
                if !set.contains(&i.endpoint) {
                    anyhow::bail!("Endpoint {} does not exist on interface {} on namespace {}", i.endpoint, i.name, n.name);
                }
            }
        }
    }

    if let Some(con) = &c.connections {
        for c in con {
            if !set.contains(&c.a) {
                anyhow::bail!("Endpoint {} does not exist on connection {}", c.a, c.name);
            }
            if !set.contains(&c.b) {
                anyhow::bail!("Endpoint {} does not exist on connection {}", c.b, c.name);
            }
        }
    }


    // Specific checks
    if let Some(ns) = &c.namespace {
        for n in ns {
            n.checks().context(format!("Checks failed for namespace {}", n.name))?;
        }
    }

    Ok(())
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
