use std::path::PathBuf;
use super::{MGMT_FILE_NAME, PID_FILE_NAME, SOCK_FILE_NAME};

/// This is the internal rappresentation of a switch

pub struct Switch {
    /// The name should be unique
    name: String,
}

impl Switch {

    pub fn new(name: String) -> Switch {
        Switch { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get base path of all the files related to the switch give
    /// the global base path
    pub fn base_path(&self, base: PathBuf) -> PathBuf {
        base.join(&self.name)
    }

    pub fn pid_path(&self, base: PathBuf) -> PathBuf {
        self.base_path(base).join(PID_FILE_NAME)
    }

    pub fn mgmt_path(&self, base: PathBuf) -> PathBuf {
        self.base_path(base).join(MGMT_FILE_NAME)
    }

    pub fn sock_path(&self, base: PathBuf) -> PathBuf {
        self.base_path(base).join(SOCK_FILE_NAME)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_switch() {
        let name = "test";
        let sw = Switch::new(name.to_owned());

        assert_eq!(sw.get_name(), name);
    }
}
