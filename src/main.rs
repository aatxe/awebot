#![feature(dynamic_lib, fs_walk, path_ext, unboxed_closures)]
extern crate irc;

use std::collections::HashMap;
use std::dynamic_lib::DynamicLibrary;
use std::ffi::OsStr;
use std::fmt::{Debug, Error, Formatter};
use std::fs::walk_dir;
use std::io::{BufReader, BufWriter, Result};
use std::io::prelude::*;
use std::path::Path;
use std::result::Result as StdResult;
use std::thread::spawn;
#[cfg(windows)] use std::os::windows::fs::MetadataExt;
#[cfg(unix)] use std::os::unix::fs::MetadataExt;
use irc::client::conn::NetStream;
use irc::client::prelude::*;

fn main() {
    let guards: Vec<_> = walk_dir(".").unwrap().flat_map(|p| {
        let path = p.unwrap().path();
        println!("path: {:?}", path);
        path.clone().extension().map(|ext| match ext.to_str() {
            Some("json") => Some(Config::load(path).unwrap()),
            _ => None,
        }).into_iter().filter(|c| c.is_some()).map(|c| c.unwrap())
    }).map(|config| {
        spawn(|| {
            let server = IrcServer::from_config(config).unwrap();
            loop {
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
                server.reconnect().unwrap();
            }
        })
    }).collect();
    guards.into_iter().map(|h| h.join().unwrap()).count();
}

type NetServer<'a> = ServerExt<'a, BufReader<NetStream>, BufWriter<NetStream>>;

struct Function<'a> {
    _lib: DynamicLibrary,
    pub process: fn(&'a NetServer<'a>, Message) -> Result<()>,
    pub modified: u64,
}

impl<'a> Debug for Function<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> StdResult<(), Error> {
        write!(fmt, "fn (server, message) -> IoResult<()> : {}", self.modified)
    }
}

#[cfg(windows)]
fn modified(path: &Path) -> Result<u64> {
    Ok(try!(path.metadata()).last_write_time())
}

#[cfg(unix)]
fn modified(path: &Path) -> Result<u64> {
    Ok(try!(path.metadata()).mtime_nsec() as u64)
}

fn process_message_dynamic<'a>(server: &'a NetServer<'a>, message: Message,
                               cache: &mut HashMap<String, Function<'a>>) -> Result<()> {
    let valid: [&OsStr; 3] = ["dylib".as_ref(), "so".as_ref(), "dll".as_ref()];
    for path in walk_dir("plugins/").unwrap() {
        let path = try!(path).path();
        if path.extension().is_none() || !valid.contains(&path.extension().unwrap()) {
            continue
        }
        let modified = try!(modified(&path));
        let key = path.clone().into_os_string().into_string().unwrap();
        if !cache.contains_key(&key) || cache[&key].modified != modified {
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
        try!((cache[&key].process)(server, message.clone()));
    }
    Ok(())
}
