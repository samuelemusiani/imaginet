use std::path::PathBuf;

pub struct Namespace {
    name: String,
    interfaces: Vec<NSInterface>,
}

pub struct NSInterface {
    name: String,
    ip: String,
    /// relative path to the vde endpoint. The base path is not given
    /// because it depends on the executor
    endpoint: String,
    port: Option<u32>,
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

    pub fn exec_command(&self) -> String {
        "vdens".to_owned()
    }

    pub fn exec_args(&self, base: &str, starter: &str) -> Vec<String> {
        if self.interfaces.len() == 0 {
            return vec!();
        } 

        let mut args = vec!("--multi".to_owned());
        let b = PathBuf::from(base);
        for i in &self.interfaces {
            let p = b.join(&i.endpoint).to_str().unwrap().to_owned();
            if let Some(port) = i.port {
                args.push(format!("{p}[{port}]"));
            } else {
                args.push(p);
            }
        }
        args.push("--".to_owned());
        args.push(starter.to_owned());
        args.push(base.to_owned());
        args.push(self.name.to_owned());
        return args;
    }
}

impl NSInterface {
    pub fn new(name: String, ip: String, endpoint: String, 
        port: Option<u32>) -> NSInterface {
        NSInterface {
            name,
            ip,
            endpoint,
            port,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_ip(&self) -> &String {
        &self.ip
    }

    pub fn get_endpoint(&self) -> &String {
        &self.endpoint
    }

    pub fn get_port(&self) -> Option<u32> {
        self.port
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
