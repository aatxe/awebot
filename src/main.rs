#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate failure;
#[macro_use]
extern crate log;
extern crate irc;

type Result<T> = ::std::result::Result<T, ::failure::Error>;

fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    while let Err(e) = main_impl() {
        let report = e.causes().skip(1).fold(format!("{}", e), |acc, err| {
            format!("{}: {}", acc, err)
        });
        error!("{}", report);
        info!("{}", e.backtrace());
    }
}

fn main_impl() -> Result<()> {
    println!("Hello, world!");
    Ok(())
}
