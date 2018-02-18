use irc::client::prelude::*;

use config::Config;
use cmd::*;
use dispatch;
use dispatch::Dispatcher;
use error::*;

pub fn main_impl() -> Result<()> {
    let config = Config::load("merveille.toml")?;
    let dispatcher = dispatcher!(',', Quit);

    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&config.into())?;
    client.identify()?;

    reactor.register_client_with_handler(client, move |client, message| {
        trace!("{}", message.to_string().trimmed());
        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if let Some(source) = message.source_nickname() {
                dispatcher.dispatch(
                    &client, source, message.response_target().unwrap_or(target), msg
                )?;
            } else {
                warn!("received PRIVMSG without source");
                warn!("in: {}", message.to_string().trimmed());
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
