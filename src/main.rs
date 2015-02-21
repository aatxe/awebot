#![feature(old_io, old_path, std_misc, unboxed_closures)]
extern crate irc;

use std::borrow::ToOwned;
use std::collections::HashMap;
use std::dynamic_lib::DynamicLibrary;
use std::fmt::{Debug, Error, Formatter};
use std::old_io::{BufferedReader, BufferedWriter, IoResult};
use std::old_io::fs::{PathExtensions, walk_dir};
use irc::client::conn::NetStream;
use irc::client::data::Message;
use irc::client::server::{IrcServer, Server};
use irc::client::server::utils::Wrapper;

fn main() {
    loop {
        let irc_server = IrcServer::new("config.json").unwrap();
        let server = Wrapper::new(&irc_server);
        server.identify().unwrap();
        let mut cache = HashMap::new();
        for message in server.iter() {
            match message {
                Ok(message) => {
                    print!("{}", message.into_string());
                    process_message_dynamic(&server, message, &mut cache).unwrap();
                },
                Err(e) => {
                    println!("Reconnecting because {}", e);
                    break
                }
            }
        }
    }
}

type NetWrapper<'a> = Wrapper<'a, BufferedReader<NetStream>, BufferedWriter<NetStream>>;

struct Function<'a> { 
    _lib: DynamicLibrary,
    pub process: fn(&'a NetWrapper<'a>, Message) -> IoResult<()>,
    pub modified: u64,
}

impl<'a> Debug for Function<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "fn (server, message) -> IoResult<()> : {}", self.modified)
    }
}

fn process_message_dynamic<'a>(server: &'a NetWrapper<'a>, message: Message, 
                               cache: &mut HashMap<String, Function<'a>>) -> IoResult<()> {
    let valid = [b"dylib", b"so", b"dll"];
    for path in walk_dir(&Path::new("plugins/")).unwrap() {
        if path.extension().is_none() || !valid.contains(&path.extension().unwrap()) { continue }
        let modified = path.stat().unwrap().modified;
        let key = path.as_str().unwrap().to_owned();
        if !cache.contains_key(&key) || cache[key].modified != modified {
            cache.remove(&key);
            let lib = DynamicLibrary::open(Some(&path)).unwrap();   
            let func = Function { 
                process: unsafe {
                    std::mem::transmute(lib.symbol::<u8>("process").unwrap())
                },
                _lib: lib,
                modified: modified,
            };
            cache.insert(key.clone(), func);
        }
        try!((cache[key].process)(server, message.clone()));    
    }
    Ok(())
}
