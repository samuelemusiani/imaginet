use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Switch {
    pub name: String,
    pub vdeterm: bool,
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
    pub switch: Vec<Switch>,
    pub namespace: Vec<Namespace>,
    pub connections: Vec<Connection>
}

