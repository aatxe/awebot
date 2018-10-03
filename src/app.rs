use std::rc::Rc;
use std::time::Duration;

use clap::{Arg, App};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use failure::err_msg;
use irc::client::prelude::*;
use irc::error::IrcError::Timer;
use tokio_timer::wheel;

use cmd::*;
use dispatch::Dispatcher;
use error::*;

// Embed Diesel migrations.
embed_migrations!();

pub fn main_impl() -> Result<()> {
    let clap = App::new("awebot")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Aaron Weiss <awe@pdgn.co>")
        .about("a lovely IRC bot")
        .arg(Arg::with_name("config").help("Configuration file for awebot").required(true).index(1))
        .get_matches();

    let config = Config::load(
        clap.value_of("config").unwrap()
    )?;
    let db_path = config.get_option("database").ok_or_else(|| {
        Permanent(err_msg("must specify a database path in the configuration"))
    })?;
    let () = embedded_migrations::run(&SqliteConnection::establish(db_path)?).map_err(|e| {
        Permanent(DatabaseSetupFailed {
            database: db_path.to_owned(),
            cause: e,
        }.into())
    })?;
    let whois = Rc::new(Whois::from(SqliteConnection::establish(db_path)?));


    let mut reactor = IrcReactor::new()?;

    let dispatcher = dispatcher!(
        '@',
        Rehash::from(config.owners.clone().unwrap_or_else(Vec::new)),
        Tell::from(SqliteConnection::establish(db_path)?),
        IAm::from(SqliteConnection::establish(db_path)?),
        Whoami::from(whois.clone()),
        whois,
        SendTweet::new(&config, reactor.inner_handle()),
    );

    let client = reactor.prepare_client_and_connect(&config)?;
    client.identify()?;

    reactor.register_client_with_handler(client.clone(), move |client, message| {
        trace!("{}", message.to_string().trimmed());

        if let Command::PRIVMSG(ref target, ref msg) = message.command {
            if let Some(source) = message.source_nickname() {
                dispatcher.dispatch(
                    client, source, message.response_target().unwrap_or(target), msg
                )?;
            } else {
                warn!("received PRIVMSG without source");
                warn!("in: {}", message.to_string().trimmed());
            }
        }
        Ok(())
    });

    let who_interval = wheel()
        .tick_duration(Duration::from_secs(1))
        .num_slots(256)
        .build()
        .interval(Duration::from_secs(20));

    reactor.register_future(who_interval.map_err(Timer).for_each(move |()| {
            for chan in client.list_channels().expect("unreachable") {
                client.send(Command::WHO(Some(chan.to_owned()), None))?;
            }
            Ok(())
        }
    ));

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
