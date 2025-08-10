use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_port: u16,
}
