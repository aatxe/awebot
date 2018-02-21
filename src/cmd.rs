use std::collections::HashSet;

use chrono::Utc;
use diesel;
use diesel::prelude::*;
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
    fn command(&self) -> &'static str {
        "rehash"
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
    fn command(&self) -> &'static str {
        "tell"
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
