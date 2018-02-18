#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate log;
extern crate irc;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

#[macro_use]
mod dispatch;

mod app;
mod cmd;
mod config;
mod error;

use error::*;

const VERSION_STR: &str = concat!(
    env!("CARGO_PKG_NAME"),
    ":",
    env!("CARGO_PKG_VERSION"),
    ":Compiled with rustc",
);

fn main() {
    env_logger::init();

    while let Err(err) = app::main_impl() {
        match err {
            Ephemeral(e) => report_err(e),
            Permanent(e) => {
                report_err(e);
                break;
            }
        }
    }
}

fn report_err(e: failure::Error) {
    let report = e.causes().skip(1).fold(format!("{}", e), |acc, err| {
        format!("{}: {}", acc, err)
    });
    error!("{}", report);
    info!("{}", e.backtrace());
}
