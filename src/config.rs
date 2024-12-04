use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Switch {
    pub name: String,
    pub config: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    pub name: String,
    pub connected: String,
    pub ip: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub name: String,
    pub a: String,
    pub b: String,
    pub wirefilter: Option<bool>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub switch: Option<Vec<Switch>>,
    pub namespace: Option<Vec<Namespace>>,
    pub connections: Option<Vec<Connection>>
}

impl Config {
    pub fn from_string(file: &str) -> Config {
        serde_yaml::from_str(&file).unwrap()
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
        let c = Config::from_string(file);
        let sws = c.switch.unwrap();
        assert_eq!(sws.len(), 1);
        let sw = &sws[0];
        assert_eq!(sw.name, "test");
        assert_eq!(sw.config, Some("test.conf".to_owned()));
    }
}
