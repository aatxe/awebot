use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use irc::client::prelude::*;
use irc::error::IrcError::Timer;
use irc::proto::response::Response::RPL_WHOREPLY;
use rand;
use rand::Rng;
use tokio_timer::wheel;

use config::Config;
use cmd::*;
use dispatch::Dispatcher;
use error::*;

pub fn main_impl() -> Result<()> {
    let config = Arc::new(Config::load("merveille.toml")?);
    let lobby = Arc::new(Mutex::new(HashSet::new()));
    let dispatcher = dispatcher!(',', Quit);

    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&Config::clone(&config).into())?;
    let recv_config = config.clone();
    let recv_lobby = lobby.clone();
    client.identify()?;

    reactor.register_client_with_handler(client.clone(), move |client, message| {
        let lobby = &recv_lobby;
        let config = &recv_config;

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
        } else if let Command::Response(RPL_WHOREPLY, ref args, _) = message.command {
            if args.len() == 6 && args[0] == config.lobby() { // RFC 2812
                lobby.lock().unwrap().insert(args[4].to_owned());
            } else if args.len() == 7 && args[1] == config.lobby() { // InspIRCd
                lobby.lock().unwrap().insert(args[5].to_owned());
            } else {
                warn!("received unusual RPL_WHOREPLY");
                warn!("in: {}", message.to_string().trimmed());
            }
        } else if let Command::JOIN(ref chan, _, _) = message.command {
            if chan == config.lobby() {
                if let Some(source) = message.source_nickname() {
                    lobby.lock().unwrap().insert(source.to_owned());
                }
            }
        }
        Ok(())
    });

    let who_interval = wheel()
        .tick_duration(Duration::from_secs(1))
        .num_slots(256)
        .build()
        .interval(Duration::from_secs(20));
    let who_client = client.clone();
    let who_config = config.clone();

    reactor.register_future(who_interval.map_err(|e| Timer(e)).for_each(move |()| {
        let client = &who_client;
        let config = &who_config;
        client.send(Command::WHO(Some(config.lobby().to_owned()), None))
    }));

    let reward_interval = wheel()
        .tick_duration(Duration::from_secs(1))
        .num_slots(4096)
        .build()
        .interval(Duration::from_secs(u64::from(config.reward_time())));
    let reward_client = client.clone();
    let reward_config = config.clone();

    reactor.register_future(reward_interval.map_err(|e| Timer(e)).for_each(move |()| {
        let client = &reward_client;
        let config = &reward_config;
        let mut rng = rand::thread_rng();

        for user in lobby.lock().unwrap().iter() {
            if rng.gen_weighted_bool(config.reward_rate()) {
                client.send_privmsg(
                    config.lobby(), format!("would've issued a reward to {}", user)
                )?;
            } else {
                client.send_privmsg(
                    config.lobby(), format!("did not issue a reward to {}", user)
                )?;
            }
        }
        Ok(())
    }));

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
