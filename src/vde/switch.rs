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

    /// Get base path of all the files related to the switch given
    /// the global base path
    pub fn base_path(&self, base: PathBuf) -> PathBuf {
        base.join(&self.name)
    }

    /// Get the path of the pid file of the switch given the global base path
    pub fn pid_path(&self, base: PathBuf) -> PathBuf {
        self.base_path(base).join(PID_FILE_NAME)
    }

    /// Get the path of the management file of the switch given the global base path
    pub fn mgmt_path(&self, base: PathBuf) -> PathBuf {
        self.base_path(base).join(MGMT_FILE_NAME)
    }

    /// Get the path of the socket file of the switch given the global base path
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

    #[test]
    fn switch_base_path() {
        let name = "lara";
        let sw = Switch::new(name.to_owned());
        let base = PathBuf::from("/tmp");

        assert_eq!(sw.base_path(base), PathBuf::from("/tmp/lara"));
    }

    #[test]
    fn switch_pid_path() {
        let name = "maasldkf";
        let sw = Switch::new(name.to_owned());
        let base = PathBuf::from("/tmp");

        assert_eq!(sw.pid_path(base), PathBuf::from("/tmp/maasldkf/pid"));
    }

    #[test]
    fn switch_mgmt_path() {
        let name = "sdfk3i";
        let sw = Switch::new(name.to_owned());
        let base = PathBuf::from("/tmp");

        assert_eq!(sw.mgmt_path(base), PathBuf::from("/tmp/sdfk3i/mgmt"));
    }

    #[test]
    fn switch_sock_path() {
        let name = "sw-13ndo28";
        let sw = Switch::new(name.to_owned());
        let base = PathBuf::from("/tmp");

        assert_eq!(sw.sock_path(base), PathBuf::from("/tmp/sw-13ndo28/sock"));
    }
}
