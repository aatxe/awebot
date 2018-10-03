use diesel_migrations::RunMigrationsError;
use failure;

pub type Result<T> = ::std::result::Result<T, Error>;

/// A wrapper around `failure::Error` that records whether or not the error is permanent.
/// Errors will automatically be treated as `Ephemeral` via the `From` conversion.
#[derive(Debug)]
pub enum Error {
    /// A temporary error that can safely be retried.
    Ephemeral(failure::Error),
    /// A permanent error that should cause the program to exit.
    Permanent(failure::Error),
}

impl<F: failure::Fail> From<F> for Error {
    fn from(fail: F) -> Error {
        Ephemeral(fail.into())
    }
}

#[derive(Debug, Fail)]
#[fail(display = "failed to configure database: {}", database)]
pub struct DatabaseSetupFailed {
    pub database: String,
    #[fail(cause)]
    pub cause: RunMigrationsError,
}

pub use self::Error::{Ephemeral, Permanent};
