#![feature(if_let, slicing_syntax)]
extern crate irc;

use std::collections::HashMap;
use std::dynamic_lib::DynamicLibrary;
use std::fmt::{Error, Formatter, Show};
use std::io::{BufferedStream, IoResult};
use std::io::fs::{PathExtensions, walk_dir};
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
    let mut cache = HashMap::new();
    for message in server.iter() {
        print!("{}", message.into_string());
        process_message_dynamic(&server, message, &mut cache).unwrap();
    }
}

struct Function<'a> { 
    lib: DynamicLibrary,
    pub process: Option<fn(&'a Wrapper<'a, BufferedStream<NetStream>>, Message) -> IoResult<()>>,
    pub modified: u64,
}

impl<'a> Function<'a> {
    fn eval(&mut self) {
        self.process = Some(unsafe {
            std::mem::transmute(self.lib.symbol::<u8>("process").unwrap())
        });
    }
}

impl<'a> Show for Function<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        try!("fn (server, message) -> IoResult<()> : ".fmt(fmt));
        self.modified.fmt(fmt)
    }
}

fn process_message_dynamic<'a>(server: &'a Wrapper<'a, BufferedStream<NetStream>>,
                               message: Message, cache: &mut HashMap<String, Function<'a>>) 
-> IoResult<()> {
    for path in walk_dir(&Path::new("plugins/")).unwrap() {
        if path.extension().is_none() || path.extension().unwrap() != b"dylib" &&
           path.extension().unwrap() != b"so" { continue }
        let modified = path.stat().unwrap().modified;
        let key = path.as_str().unwrap().into_string();
        if !cache.contains_key(&key) || cache[key].modified != modified {
            let lib = DynamicLibrary::open(Some(path.as_str().unwrap())).unwrap();   
            let mut func = Function { 
                lib: lib,
                process: None,
                modified: modified,
            };
            func.eval();
            cache.insert(key.clone(), func);
        }
        try!((cache[key].process.unwrap())(server, message.clone()));    
    }
    Ok(())
}
