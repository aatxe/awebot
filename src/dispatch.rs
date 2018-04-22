use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use irc::client::IrcClient;
use irc::error::Result;

#[derive(Copy, Clone)]
pub struct Context<'a> {
    pub client: &'a IrcClient,
    pub sender: &'a str,
    pub respond_to: &'a str,
    pub args: &'a [&'a str],
    pub msg: &'a str,
}

pub trait Handler {
    fn command(&self) -> &'static [&'static str];

    fn handle<'a>(&self, context: Context<'a>) -> Result<()>;

    fn on_each_message<'a>(&self, _: Context<'a>) -> Result<()> {
        Ok(())
    }
}

impl<T> Handler for Rc<T> where T: Handler {
    fn command(&self) -> &'static [&'static str] {
        self.deref().command()
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        self.deref().handle(context)
    }

    fn on_each_message<'a>(&self, context: Context<'a>) -> Result<()> {
        self.deref().on_each_message(context)
    }
}

impl<T> Handler for Arc<T> where T: Handler {
    fn command(&self) -> &'static [&'static str] {
        self.deref().command()
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        self.deref().handle(context)
    }

    fn on_each_message<'a>(&self, context: Context<'a>) -> Result<()> {
        self.deref().on_each_message(context)
    }
}

impl<T> Handler for Option<T> where T: Handler {
    fn command(&self) -> &'static [&'static str] {
        match self {
            &Some(ref handler) => handler.command(),
            &None => &[]
        }
    }

    fn handle<'a>(&self, context: Context<'a>) -> Result<()> {
        match self {
            &Some(ref handler) => handler.handle(context),
            &None => Ok(())
        }
    }

    fn on_each_message<'a>(&self, context: Context<'a>) -> Result<()> {
        match self {
            &Some(ref handler) => handler.on_each_message(context),
            &None => Ok(())
        }
    }
}

pub struct Dispatcher {
    line_start: char,
    handlers: Vec<Box<Handler>>,
    cmd_map: HashMap<&'static str, usize>,
}

impl Dispatcher {
    pub fn new(line_start: char) -> Dispatcher {
        Dispatcher {
            line_start: line_start,
            handlers: Vec::new(),
            cmd_map: HashMap::new(),
        }
    }

    pub fn register<H>(&mut self, handler: H) where H: Handler + 'static {
        for cmd in handler.command() {
            self.cmd_map.insert(cmd, self.handlers.len());
        }
        self.handlers.push(Box::new(handler));
    }

    pub fn get_handler(&self, command: &str) -> Option<&Box<Handler>> {
        self.cmd_map.get(command).map(|idx| &self.handlers[*idx])
    }

    pub fn dispatch<'a>(
        &self, client: &IrcClient, sender: &str, respond_to: &str, message: &str,
    ) -> Result<()> {
        if !message.starts_with(self.line_start) {
            for handler in &self.handlers {
                handler.on_each_message(Context {
                    client: client,
                    sender: sender,
                    respond_to: respond_to,
                    args: &[],
                    msg: message,
                })?;
            }
            return Ok(())
        }

        let message = &message[1..];
        let fragments: Vec<_> = message.split(' ').collect();
        if fragments.len() == 0 {
            return Ok(())
        }

        let command = fragments[0];
        let context = Context {
            client: client,
            sender: sender,
            respond_to: respond_to,
            args: &fragments[1..],
            msg: message,
        };


        self.get_handler(&command).map(|handler| handler.handle(context)).unwrap_or(Ok(()))
    }
}

#[macro_export]
macro_rules! dispatcher {
    ( $s:expr ) => (Dispatcher::new($s));
    ( $s:expr, $( $x:expr ),* $(,)* ) => {
        {
            let mut temp_dispatcher = Dispatcher::new($s);
            $(
                temp_dispatcher.register($x);
            )*
            temp_dispatcher
        }
    };
}
