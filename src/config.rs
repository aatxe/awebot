use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use failure;
use irc::client::data::Config as IrcConfig;
use toml;

use error::*;

#[derive(Clone, Deserialize, Default, Debug, Serialize)]
pub struct Config {
    // Required
    server: String,
    lobby: String,

    // Optional
    nickname: Option<String>,
    port: Option<u16>,
    ssl: Option<bool>,
    database: Option<String>,
}

impl From<Config> for IrcConfig {
    fn from(config: Config) -> IrcConfig {
        IrcConfig {
            nickname: config.nickname.or_else(|| Some(env!("CARGO_PKG_NAME").to_owned())),
            server: Some(config.server),
            port: config.port,
            use_ssl: config.ssl,
            channels: Some(vec![config.lobby]),
            umodes: Some("+B".to_owned()),
            user_info: Some("merveille: an IRC collect-and-combat game.".to_owned()),
            version: Some(::VERSION_STR.to_owned()),
            ..IrcConfig::default()
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
        let contents = File::open(&path).and_then(|mut file| {
            let mut buf = String::new();
            file.read_to_string(&mut buf).map(|_| buf)
        }).map_err(move |err| {
            let err: failure::Error = err.into();
            Permanent(err.context(
                format!("{}", path.as_ref().display())
            ).into())
        })?;

        Ok(toml::from_str(&contents[..])?)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut file = File::create(&path)?;
        let contents = toml::to_string(self)?;
        file.write_all(contents.as_bytes())?;
        Ok(())
    }

    pub fn server(&self) -> &str {
        &self.server
    }

    pub fn lobby(&self) -> &str {
        &self.lobby
    }

    pub fn nickname(&self) -> &str {
        self.database.as_ref().map(|s| &s[..]).unwrap_or(env!("CARGO_PKG_NAME"))
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(6667)
    }

    pub fn ssl(&self) -> bool {
        self.ssl.unwrap_or_default()
    }

    pub fn database(&self) -> &str {
        self.database.as_ref().map(|s| &s[..]).unwrap_or(concat!(env!("CARGO_PKG_NAME"), ".db"))
    }
}
