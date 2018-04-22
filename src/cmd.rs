use std::collections::HashSet;
use std::rc::Rc;

use chrono::Utc;
use diesel;
use diesel::prelude::*;
use diesel::result::{Error as QueryError};
use diesel::sqlite::SqliteConnection;
use irc::client::prelude::*;
use irc::error::Result;
use irc::error::IrcError::Custom;

use dispatch::{Context, Handler};

pub struct Rehash {
    allowed: HashSet<String>,
}

impl From<Vec<String>> for Rehash {
    fn from(allowed: Vec<String>) -> Rehash {
        Rehash {
            allowed: allowed.into_iter().collect(),
        }
    }
}

impl Handler for Rehash {
    fn command(&self) -> &'static [&'static str] {
        &["rehash"]
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        if self.allowed.contains(context.sender) {
            context.client.send_quit(format!("Quitting due to command from {}", context.sender))
        } else {
            Ok(())
        }
    }
}

pub struct Tell {
    conn: SqliteConnection,
}


impl From<SqliteConnection> for Tell {
    fn from(conn: SqliteConnection) -> Tell {
        Tell {
            conn: conn,
        }
    }
}

impl Handler for Tell {
    fn command(&self) -> &'static [&'static str] {
        &["tell"]
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        use models::*;
        use schema::mail;

        if context.args.len() < 2 {
            return Ok(());
        }

        let target = context.args[0];
        if target == context.client.current_nickname() {
            return context.client.send_privmsg(context.respond_to, "I'm right here!");
        }

        let new_message = NewMessage {
            target: target,
            sender: context.sender,
            message: &context.args[1..].join(" "),
            sent: &Utc::now().naive_utc(),
            // messages should be private if they were sent in queries
            private: context.respond_to == context.sender,
        };

        diesel::insert_into(mail::table)
            .values(&new_message)
            .execute(&self.conn)
            .map_err(|e| Custom { inner: e.into() })?;

        context.client.send_privmsg(
            context.respond_to, format!("{}: I'll let them know!", context.sender)
        )?;

        Ok(())
    }

    fn on_each_message<'a>(&self, context: Context<'a>) -> Result<()> {
        use models::Message;
        use schema::mail::dsl::*;

        let results = mail
            .filter(target.eq(context.sender))
            .load::<Message>(&self.conn)
            .map_err(|e| Custom { inner: e.into() })?;

        for msg in results {
            if msg.private {
                context.client.send_privmsg(context.sender, format!("{}", msg))?;
            } else {
                context.client.send_privmsg(context.respond_to, format!("{}", msg))?;
            }
        }

        diesel::delete(
            mail.filter(target.eq(context.sender))
        ).execute(&self.conn).map_err(|e| Custom { inner: e.into() })?;

        Ok(())
    }
}

pub struct IAm {
    conn: SqliteConnection,
}

impl From<SqliteConnection> for IAm {
    fn from(conn: SqliteConnection) -> IAm {
        IAm {
            conn: conn,
        }
    }
}

impl Handler for IAm {
    fn command(&self) -> &'static [&'static str] {
        &["iam"]
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        use models::*;
        use schema::whois;

        if context.args.len() == 0 {
            return context.client.send_privmsg(
                context.respond_to, format!(
                    "{}: Who are you? Let me know by writing a description after the command!",
                    context.sender,
                )
            );
        }

        let description = context.args.join(" ");

        let new_whois = NewWhoisEntry {
            nickname: context.sender,
            description: &description,
        };

        diesel::insert_into(whois::table)
            .values(&new_whois)
            .execute(&self.conn)
            .map_err(|e| Custom { inner: e.into() })?;

        context.client.send_privmsg(
            context.respond_to, format!("{}: Got it!", context.sender)
        )?;

        Ok(())
    }
}

pub struct Whois {
    conn: SqliteConnection,
}

impl From<SqliteConnection> for Whois {
    fn from(conn: SqliteConnection) -> Whois {
        Whois {
            conn: conn,
        }
    }
}

impl Handler for Whois {
    fn command(&self) -> &'static [&'static str] {
        &["whois", "whodat"]
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        use models::*;
        use schema::whois::dsl::*;

        if context.args.len() == 0 {
            return context.client.send_privmsg(
                context.respond_to, format!(
                    "{}: Who do you want to knouw about? Let me know by writing their nickname \
                     after the command!",
                    context.sender,
                )
            );
        }

        for nick in context.args {
            let msg = match whois.find(nick).first::<WhoisEntry>(&self.conn) {
                Ok(res) => if res.nickname == context.sender {
                    format!(
                        "{}: you are {}", context.sender, res.description
                    )
                } else {
                    format!(
                        "{}: {} is {}", context.sender, res.nickname, res.description
                    )
                },
                Err(QueryError::NotFound) => if *nick == context.sender {
                    format!(
                        "{}: I don't know who you are. Why don't you tell me about yourself with \
                         iam?", context.sender
                    )
                } else {
                    format!(
                        "{}: I don't know who {} is.", context.sender, nick
                    )
                },
                Err(e) => return Err(Custom { inner: e.into() }),
            };

            context.client.send_privmsg(context.respond_to, msg)?;
        }

        Ok(())
    }
}

pub struct Whoami {
    whois: Rc<Whois>,
}

impl From<Rc<Whois>> for Whoami {
    fn from(whois: Rc<Whois>) -> Whoami {
        Whoami { whois }
    }
}

impl Handler for Whoami {
    fn command(&self) -> &'static [&'static str] {
        &["whoami"]
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        self.whois.handle(Context {
            args: &[context.sender],
            .. context
        })
    }
}
