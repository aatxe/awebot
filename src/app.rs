use irc::client::prelude::*;

use config::Config;
use error::*;

pub fn main_impl() -> Result<()> {
    let config = Config::load("merveille.toml")?;

    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&config.into())?;
    client.identify()?;

    reactor.register_client_with_handler(client, |client, message| {
        trace!("{}", message.to_string().trimmed());
        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if msg == "!quit" {
                client.send_quit(format!(
                    "Quitting: command from {}", message.source_nickname().unwrap_or(target)
                ))?;
            }
        }
        Ok(())
    });

    reactor.run()?;
    Ok(())
}

trait StringTrim {
    fn trimmed(self) -> Self;
}

impl StringTrim for String {
    fn trimmed(mut self) -> Self {
        if self.ends_with('\n') {
            self.pop();
        }
        if self.ends_with('\r') {
            self.pop();
        }
        self
    }
}
