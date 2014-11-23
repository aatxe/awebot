#![feature(if_let, slicing_syntax)]
extern crate irc;

use std::collections::HashMap;
use std::dynamic_lib::DynamicLibrary;
use std::io::{BufferedStream, IoResult};
use std::io::fs::walk_dir;
use irc::conn::NetStream;
use irc::data::{Config, Message};
use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

fn main() {
    let config = Config {
        owners: vec!("awe".into_string()),
        nickname: "awebot".into_string(),
        username: "awebot".into_string(),
        realname: "awebot".into_string(),
        password: "".into_string(),
        server: "irc.fyrechat.net".into_string(),
        port: 6667,
        use_ssl: false,
        channels: vec!("#vana".into_string()),
        options: HashMap::new(),
    };
    let irc_server = IrcServer::from_config(config).unwrap();
    let server = Wrapper::new(&irc_server);
    server.identify().unwrap();
    for message in server.iter() {
        print!("{}", message.into_string());
        process_message_dynamic(&server, message).unwrap();
    }
}

fn process_message_dynamic<'a>(server: &'a Wrapper<'a, BufferedStream<NetStream>>, 
                               message: Message) -> IoResult<()> {
    for path in walk_dir(&Path::new("plugins/")).unwrap() {
        if path.extension().is_none() || path.extension().unwrap() != b"dylib" { continue }
        let lib = DynamicLibrary::open(Some(path.as_str().unwrap())).unwrap();            
        let process: fn(&'a Wrapper<'a, BufferedStream<NetStream>>, Message) -> IoResult<()> = 
            unsafe {
            std::mem::transmute(lib.symbol::<u8>("process").unwrap())
        };
        try!(process(server, message.clone()));
    }
    Ok(())
}
