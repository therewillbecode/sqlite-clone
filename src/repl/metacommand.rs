use anyhow::{anyhow, bail, Result};
use derive_more::Display;
use log::debug;
use std::str::FromStr;

#[derive(Debug, Display)]
enum Metacommand {
    Tables,
    Schema,
    Indexes,
}

impl FromStr for Metacommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        debug!("{}", s);
        match s {
            ".tables" => Ok(Metacommand::Tables),
            ".schema" => Ok(Metacommand::Schema),
            ".indexes" => Ok(Metacommand::Indexes),
            _ => bail!("Failed to parse metacommand"),
        }
    }
}

pub fn handle_metacommand(cmd: &str) -> Result<String> {
    let cmd: Metacommand = Metacommand::from_str(cmd)?;
    return Ok(cmd.to_string());
}
