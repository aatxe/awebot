use irc::client::IrcClient;
use irc::error::Result;

pub struct Context<'a> {
    pub client: &'a IrcClient,
    pub sender: &'a str,
    pub respond_to: &'a str,
    pub args: &'a [&'a str],
}

pub trait Handler {
    fn command(&self) -> &'static str;

    fn handle<'a>(&self, context: Context<'a>) -> Result<()>;
}

pub struct Dispatcher {
    line_start: char,
    handlers: Vec<Box<Handler>>,
}

impl Dispatcher {
    pub fn new(line_start: char) -> Dispatcher {
        Dispatcher {
            line_start: line_start,
            handlers: Vec::new(),
        }
    }

    pub fn register<H>(&mut self, handler: H) where H: Handler + 'static {
        self.handlers.push(Box::new(handler));
    }

    pub fn dispatch<'a>(
        &self, client: &IrcClient, sender: &str, respond_to: &str, message: &str,
    ) -> Result<()> {
        if !message.starts_with(self.line_start) {
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
        };

        for handler in &self.handlers {
            if command == handler.command() {
                handler.handle(context)?;
                break;
            }
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! dispatcher {
    ( $s:expr ) => (Dispatcher::new($s));
    ( $s:expr, $( $x:expr ),* ) => {
        {
            let mut temp_dispatcher = Dispatcher::new($s);
            $(
                temp_dispatcher.register($x);
            )*
            temp_dispatcher
        }
    };
}
