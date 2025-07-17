use std::path::Path;

use anyhow::Result;
use serde_derive::Deserialize;
use tokio::fs;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub tailscale_unit_id: String,
    pub tailscale_interface_name: String,
    pub tailscale_route_table_id: u32,
    pub journal_online_str: String,
    pub vpn_route_table_id: u32,
    pub vpn_route_fwmark: u32,
}

pub async fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    Ok(toml::from_slice(&fs::read(path).await?)?)
}
