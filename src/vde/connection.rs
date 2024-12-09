use super::PID_FILE_NAME;
use std::path::PathBuf;

pub struct Connection {
    pub name: String,
    pub a: String,
    pub port_a: Option<u32>,
    pub b: String,
    pub port_b: Option<u32>,
}

impl Connection {
    pub fn new(name: String, a: String, port_a: Option<u32>, b: String, port_b: Option<u32>) -> Connection {
        Connection {
            name,
            a,
            port_a,
            b,
            port_b,
        }
    }

    pub fn base_path(&self, base: &str) -> String {
        PathBuf::from(base).join(&self.name).to_str().unwrap().to_owned()
    }

    pub fn pid_path(&self, base: &str) -> String {
        PathBuf::from(self.base_path(base)).join(PID_FILE_NAME).to_str().unwrap().to_owned()
    }

    pub fn exec_command(&self) -> String {
        String::from("vde_plug")
    }

    pub fn exec_args(&self, base: &str) -> Vec<String> {
        let b = PathBuf::from(base);
        let mut pa = b.join(&self.a).to_str().unwrap().to_owned();
        let mut pb = b.join(&self.b).to_str().unwrap().to_owned();

        if let Some(port) = self.port_a {
            pa.push_str(&format!("[{port}]"));
        }

        if let Some(port) = self.port_b {
            pb.push_str(&format!("[{port}]"));
        }

        return vec!(
            pa, pb,
            "--pidfile".to_owned(), self.pid_path(base),
            "--descr".to_owned(), self.name.to_owned(),
            "--daemon".to_owned()
        );
    }
}
