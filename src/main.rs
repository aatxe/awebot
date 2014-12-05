#![feature(if_let, slicing_syntax, unboxed_closures)]
extern crate irc;

use std::collections::HashMap;
use std::default::Default;
use std::dynamic_lib::DynamicLibrary;
use std::fmt::{Error, Formatter, Show};
use std::io::{BufferedReader, BufferedWriter, IoResult};
use std::io::fs::{PathExtensions, walk_dir};
use irc::conn::NetStream;
use irc::data::{Config, Message};
use irc::server::{IrcServer, Server};
use irc::server::utils::Wrapper;

fn main() {
    let config = Config {
        owners: Some(vec!("awe".into_string())),
        nickname: Some("awebot".into_string()),
        server: Some("irc.pdgn.co".into_string()),
        use_ssl: Some(true),
        channels: Some(vec!("#pdgn".into_string())),
        .. Default::default()
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

type NetWrapper<'a> = Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>;

struct Function<'a> { 
    pub lib: DynamicLibrary,
    pub process: fn(&'a NetWrapper<'a>, Message) -> IoResult<()>,
    pub modified: u64,
}

impl<'a> Show for Function<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        try!("fn (server, message) -> IoResult<()> : ".fmt(fmt));
        self.modified.fmt(fmt)
    }
}

fn process_message_dynamic<'a>(server: &'a NetWrapper<'a>, message: Message, 
                               cache: &mut HashMap<String, Function<'a>>) -> IoResult<()> {
    let valid = [b"dylib", b"so", b"dll"];
    for path in walk_dir(&Path::new("plugins/")).unwrap() {
        if path.extension().is_none() || !valid.contains(&path.extension().unwrap()) { continue }
        let modified = path.stat().unwrap().modified;
        let key = path.as_str().unwrap().into_string();
        if !cache.contains_key(&key) || cache[key].modified != modified {
            cache.remove(&key);
            let lib = DynamicLibrary::open(Some(path.as_str().unwrap())).unwrap();   
            let func = Function { 
                process: unsafe {
                    std::mem::transmute(lib.symbol::<u8>("process").unwrap())
                },
                lib: lib,
                modified: modified,
            };
            cache.insert(key.clone(), func);
        }
        try!((cache[key].process)(server, message.clone()));    
    }
    Ok(())
}
