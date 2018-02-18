use config::Config;
use error::*;

pub fn main_impl() -> Result<()> {
    let config = Config::load("merveille.toml")?;
    Ok(())
}
