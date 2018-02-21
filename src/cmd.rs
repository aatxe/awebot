use irc::client::prelude::*;
use irc::error::Result;

use dispatch::{Context, Handler};

pub struct Quit;

impl Handler for Quit {
    fn command(&self) -> &'static str {
        "quit"
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        context.client.send_quit(format!("Quitting: command from {}", context.sender))
    }
}
