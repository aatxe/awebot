extern crate chrono;
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate log;
extern crate irc;
extern crate toml;
extern crate tokio_timer;

#[macro_use]
mod dispatch;

mod app;
mod cmd;
mod error;
mod models;
mod schema;

use error::*;

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
