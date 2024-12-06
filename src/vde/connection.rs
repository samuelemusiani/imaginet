use super::PID_FILE_NAME;
use std::path::PathBuf;

pub struct Connection {
    pub name: String,
    pub a: String,
    pub b: String,
}

impl Connection {
    pub fn new(name: String, a: String, b: String) -> Connection {
        Connection {
            name,
            a,
            b
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
        let pa = b.join(&self.a).to_str().unwrap().to_owned();
        let pb = b.join(&self.b).to_str().unwrap().to_owned();

        return vec!(
            pa, pb,
            "--pidfile".to_owned(), self.pid_path(base),
            "--descr".to_owned(), self.name.to_owned(),
            "--daemon".to_owned()
        );
    }
}
