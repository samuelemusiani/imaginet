use anyhow::{bail, Context, Ok, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net;
use std::path::PathBuf;

const DEFAULT_SWITCH_PORTS: u32 = 32;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Endpoint {
    pub name: String,
    pub port: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Switch {
    pub name: String,
    pub config: Option<String>,
    pub ports: Option<u32>,
    pub hub: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    pub name: String,
    pub interfaces: Vec<NSInterface>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NSInterface {
    pub name: String,
    pub ip: Option<String>,
    pub gateway: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cable {
    pub name: String,
    pub endpoint_a: Endpoint,
    pub endpoint_b: Endpoint,
    pub wirefilter: Option<bool>,
    pub config: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub switch: Option<Vec<Switch>>,
    pub namespace: Option<Vec<Namespace>>,
    pub cable: Option<Vec<Cable>>,
}

impl Config {
    pub fn from_string(file: &str, relative_path: PathBuf) -> Result<Config> {
        let mut c =
            serde_yaml::from_str::<Self>(&file).context("Deserialize config file failed")?;

        c.convert_paths(relative_path)
            .context("Could not convert all paths to be relative to the main config file")?;

        c.checks().context("Config checks failed")?;

        Ok(c)
    }

    // In the main config file for a topoly some paths can be specified.
    // For example the switch config path. We need to calculate the paths
    // based on the relative_path for the main config file
    fn convert_paths(&mut self, relative_path: PathBuf) -> Result<()> {
        // For now the only paths present in the config file are
        // - Switches config files
        // - Cables config files

        if let Some(ref mut sws) = &mut self.switch {
            for sw in sws {
                if let Some(ref mut conf) = &mut sw.config {
                    let mut b = relative_path.clone();
                    b.push(conf);

                    sw.config = Some(
                        b.to_str()
                            .context(format!(
                                "Can't convert path to string in switch {}. path: {:?}",
                                sw.name, b
                            ))?
                            .to_owned(),
                    );
                }
            }
        }

        Ok(())
    }

    fn checks(&self) -> Result<()> {
        log::trace!("Running checks for config");

        // All names must be unique

        let mut set = HashSet::new();

        log::trace!("Checking namespaces's name uniqueness");
        if let Some(ns) = &self.namespace {
            for n in ns {
                log::trace!("Namespace {}", &n.name);
                if !set.insert(&n.name) {
                    anyhow::bail!("Namespace name {} is not unique", n.name);
                }

                n.checks()
                    .context(format!("Checks failed for namespace {}", n.name))?;
            }
        }

        log::trace!("Checking switches's name uniqueness");
        if let Some(sw) = &self.switch {
            for s in sw {
                log::trace!("Switch {}", &s.name);
                if !set.insert(&s.name) {
                    anyhow::bail!("Switch name {} is not unique", s.name);
                }

                s.checks()
                    .context(format!("Checks failed for switch {}", s.name))?;
            }
        }

        log::trace!("Checking cables's name uniqueness");
        if let Some(con) = &self.cable {
            for c in con {
                log::trace!("Cable {}", &c.name);
                if !set.insert(&c.name) {
                    anyhow::bail!("Cable name {} is not unique", c.name);
                }

                c.checks()
                    .context(format!("Checks failed for cable {}", c.name))?;
            }
        }

        drop(set);

        // Endpoints must exist and ports must be valid

        let mut endpoint_map = HashMap::new();
        let mut switches = HashSet::new();
        let mut namespaces = HashSet::new();

        if let Some(sw) = &self.switch {
            for s in sw {
                switches.insert(&s.name);

                // To check if the port endpoint is valid we reuse the Endpoint struct,
                // but with a different purpose for the port field. In this case, the port
                // field is used to store the number of ports of the switch.
                let ports = match s.ports {
                    Some(p) => p,
                    None => DEFAULT_SWITCH_PORTS,
                };
                endpoint_map.insert(
                    &s.name,
                    Endpoint {
                        name: s.name.clone(),
                        port: Some(ports.to_string()),
                    },
                );
            }
        }

        if let Some(nss) = &self.namespace {
            for n in nss {
                namespaces.insert(&n.name);

                for i in &n.interfaces {
                    endpoint_map.insert(
                        &n.name,
                        Endpoint {
                            name: i.name.clone(),
                            port: Some(1.to_string()),
                        },
                    );
                }
            }
        }

        // To avoid another function we use the endpoint_check closure.
        // This simply checks if the endpoint exists and if the port is valid.
        // based on the map we created before.
        let endpoint_check = |name: String, port: Option<&String>| -> Result<()> {
            let end = endpoint_map
                .get(&name)
                .ok_or_else(|| anyhow::anyhow!("Endpoint {name} does not exist"))?;

            let end_port = end
                .port
                .as_ref()
                .expect("Internal error: port field is None");

            // If name is a switch we need to check if the port number specified
            // is lower than the number of ports on the switch
            //
            // If name is a namespace interface we need to check if the
            // interface exists on the namespace
            if switches.get(&name).is_some() {
                if port.is_none() {
                    return Ok(());
                }
                let port = port.unwrap();

                let int_port: u64 = port.parse::<u64>().context(format!(
                    "Port endpoint for switch is not an integer: {port}"
                ))?;

                let int_endport: u64 = end_port.parse::<u64>().context(format!(
                    "Port endpoint for switch is not an integer: {end_port}"
                ))?;

                if int_port >= int_endport {
                    let mut s = String::new();
                    if int_port == int_endport {
                        s.push_str("\nPorts are zero-indexed, so you should decrement the port number by one :)");
                    }
                    anyhow::bail!(
                        "Port {int_port} is out of range for endpoint {name} (max {int_endport} ports){s}"
                    );
                }
            } else if namespaces.get(&name).is_some() {
                // The only check is that the port exists here.
                // Nothing needs to ben done as the previous code already
                // checked this
            } else {
                // This should never be reached
                panic!("Could not find any endpoint that matched name: {name}")
            }

            Ok(())
        };

        // We need to check if and andpoint is used more than it should. For
        // devices like switches we have a max number of ports so we need to
        // count how many endpoint reference to the switch.
        // For namespaces we have interfaces that can only be used onces, so
        // we need to use a set for that.
        let mut multi_used_map = HashMap::<&String, u32>::new();
        let mut used_set = HashSet::new();

        if let Some(con) = &self.cable {
            for c in con {
                endpoint_check(c.endpoint_a.name.clone(), c.endpoint_a.port.as_ref())
                    .context(format!("Checks failed for cable {} endpoint A", c.name))?;
                endpoint_check(c.endpoint_b.name.clone(), c.endpoint_b.port.as_ref())
                    .context(format!("Checks failed for cable {} endpoint B", c.name))?;

                for edpt in vec![&c.endpoint_a, &c.endpoint_b] {
                    // If endpoint is a switch we add it to the used map. If the
                    // port is specified we also add it to the set. For namespaces
                    // only the set is used.
                    let name = &edpt.name;
                    if switches.get(name).is_some() {
                        *multi_used_map.entry(name).or_default() += 1;
                        if let Some(port) = edpt.port.as_ref() {
                            if !used_set.insert(format!("{name}-{port}")) {
                                anyhow::bail!(
                                    "Interface {port} on device {name} is used more than once."
                                )
                            }
                        }
                    } else if namespaces.get(name).is_some() {
                        if let Some(port) = edpt.port.as_ref() {
                            if !used_set.insert(format!("{name}-{port}")) {
                                anyhow::bail!(
                                    "Interface {port} on device {name} is used more than once."
                                )
                            }
                        }
                    }
                }
            }
        }

        // Check if endpoint have finished all the ports.
        // If endpoint is a switch we need to check against the total number of
        // ports.
        // If endpoint is a namespace we must use an interface only once
        for (name, ports) in endpoint_map {
            let used = multi_used_map.get(&name).unwrap_or(&0);
            if switches.get(&name).is_some() {
                let total_ports = ports.port.unwrap();
                let total_ports = total_ports.parse::<u32>().context(format!(
                    "Can't parse total ports into an integers: {total_ports}"
                ))?;
                if *used > total_ports {
                    anyhow::bail!("Endpoint {name} has more ports used than available ({used} > {})\nYou're trying to connect to many things to {name}", total_ports);
                }
            }
        }

        Ok(())
    }
}

impl Switch {
    fn checks(&self) -> Result<()> {
        if let Some(p) = self.ports {
            if p == 0 {
                anyhow::bail!("Switch {} has 0 ports", self.name);
            }
        }

        if let Some(c) = &self.config {
            let _ = std::fs::read_to_string(c).context(format!("Reading config file {}", c))?;
        }

        Ok(())
    }
}

impl Namespace {
    fn checks(&self) -> Result<()> {
        let mut found_gateway = false;
        for i in &self.interfaces {
            i.checks()
                .context(format!("Checks failed for interface {}", i.name))?;

            if i.gateway.is_some() {
                if found_gateway {
                    bail!("Only one interface can have a gateway")
                } else {
                    found_gateway = true
                }
            }
        }

        Ok(())
    }
}

impl NSInterface {
    pub fn checks(&self) -> Result<()> {
        // Check if IP is valid in CIDR notation

        if self.ip.is_none() {
            if self.gateway.is_none() {
                return Ok(());
            } else {
                bail!("Gateway is invalid if no ip address is specified")
            }
        }

        let tmpip = self.ip.as_ref().unwrap();

        let (ip, mask) = match tmpip.find('/') {
            Some(p) => (&tmpip[..p], &tmpip[p + 1..]),
            None => anyhow::bail!("Invalid CIDR format, missing /"),
        };
        let ip = ip
            .parse::<net::IpAddr>()
            .context(format!("IP address: {}", tmpip))?;

        let mask = mask.parse::<u8>().context("Invalid mask, not a number")?;
        match ip {
            net::IpAddr::V4(_) => {
                if mask > 32 {
                    anyhow::bail!("Invalid mask, too large for IPv4 (> 32)");
                }
            }
            net::IpAddr::V6(_) => {
                if mask > 128 {
                    anyhow::bail!("Invalid mask, too large for IPv6 (> 128)");
                }
            }
        };

        if self.gateway.is_none() {
            return Ok(());
        }

        let tmpgateway = self.gateway.as_ref().unwrap();
        let gt = tmpgateway
            .parse::<net::IpAddr>()
            .context(format!("Gateway address: {}", tmpgateway))?;

        match ip {
            net::IpAddr::V4(ip) => match gt {
                net::IpAddr::V6(_) => bail!("IP is IPv4 but gateway is IPv6"),
                net::IpAddr::V4(gt) => {
                    let ip = ip.to_bits();
                    let gt = gt.to_bits();
                    let mask = u32::MAX << (32 - mask);

                    log::debug!("ip {ip} gateway {gt} mask {mask}");
                    log::debug!("ip & mask {} gateway & mask {} ", ip & mask, gt & mask);

                    if ip == gt {
                        bail!("IP and Gatway can't be the same")
                    }
                    if ip & mask != gt & mask {
                        bail!("Gateway is not in the same subnet as the ip")
                    }
                }
            },
            net::IpAddr::V6(ip) => match gt {
                net::IpAddr::V4(_) => bail!("IP is IPv4 but gateway is IPv6"),
                net::IpAddr::V6(gt) => {
                    let ip = ip.to_bits();
                    let gt = gt.to_bits();
                    let mask = u128::MAX << (128 - mask);

                    log::debug!("ip {ip} gateway {gt} mask {mask}");
                    log::debug!("ip & mask {} gateway & mask {} ", ip & mask, gt & mask);

                    if ip == gt {
                        bail!("IP and Gatway can't be the same")
                    }
                    if ip & mask != gt & mask {
                        bail!("Gateway is not in the same subnet as the ip")
                    }
                }
            },
        }

        Ok(())
    }
}

impl Cable {
    fn checks(&self) -> Result<()> {
        if let Some(c) = &self.config {
            if !self.wirefilter.unwrap_or(false) {
                anyhow::bail!("Cable has a config file but it's not a wirefilter cable",);
            }

            let _ = std::fs::read_to_string(c).context(format!("Reading config file {}", c))?;
        }

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
      hub: true
"#;
        let c = Config::from_string(file, PathBuf::new()).unwrap();
        let sws = c.switch.unwrap();
        assert_eq!(sws.len(), 1);
        let sw = &sws[0];
        assert_eq!(sw.name, "test");
        assert_eq!(sw.config, None);
        assert_eq!(sw.ports, None);
        assert_eq!(sw.hub, Some(true));
    }
}
