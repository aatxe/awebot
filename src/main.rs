extern crate irc;
extern crate libloading;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt;
use std::fs::read_dir;
use std::path::Path;
use std::thread::spawn;
#[cfg(windows)] use std::os::windows::fs::MetadataExt;
#[cfg(unix)] use std::os::unix::fs::MetadataExt;

use irc::error;
use irc::client::prelude::*;
use libloading::{Library, Symbol};

fn main() {
    let guards: Vec<_> = read_dir(".").unwrap().flat_map(|p| {
        let path = p.unwrap().path();
        path.clone().extension().map(|ext| match ext.to_str() {
            Some("json") => Some(Config::load(path).unwrap()),
            _ => None,
        }).into_iter().filter(|c| c.is_some()).map(|c| c.unwrap())
    }).map(|config| {
        spawn(|| {
            let server = IrcServer::from_config(config).unwrap();
            server.identify().unwrap();
            let mut cache = HashMap::new();
            server.for_each_incoming(|message| {
                print!("{}", message);
                process_message_dynamic(&server, message, &mut cache).unwrap();
            })
        })
    }).collect();
    guards.into_iter().map(|h| h.join().unwrap()).count();
}

struct Lib {
    lib: Library,
    pub modified: u64,
}

impl Lib {
    fn process<'a>(&'a self) -> Symbol<'a, extern fn(&IrcServer, &Message) -> error::Result<()>> {
        unsafe {
            self.lib.get(b"process").unwrap()
        }
    }
}

impl fmt::Debug for Lib {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "fn (server, message) -> IoResult<()> : {}", self.modified)
    }
}

#[cfg(windows)]
fn modified(path: &Path) -> error::Result<u64> {
    Ok(path.metadata()?.last_write_time())
}

#[cfg(unix)]
fn modified(path: &Path) -> error::Result<u64> {
    Ok(path.metadata()?.mtime_nsec() as u64)
}

fn process_message_dynamic(server: &IrcServer, message: Message,
                           cache: &mut HashMap<String, Lib>) -> error::Result<()> {
    let valid: [&OsStr; 3] = ["dylib".as_ref(), "so".as_ref(), "dll".as_ref()];
    for path in read_dir("plugins/").unwrap() {
        let path = try!(path).path();
        if path.extension().is_none() || !valid.contains(&path.extension().unwrap()) {
            continue
        }
        let modified = try!(modified(&path));
        let key = path.clone().into_os_string().into_string().unwrap();
        if !cache.contains_key(&key) || cache[&key].modified != modified {
            cache.remove(&key);
            let lib = Library::new(path).unwrap();
            let func = Lib {
                lib: lib,
                modified: modified,
            };
            cache.insert(key.clone(), func);
        }
        try!((cache[&key].process())(server, &message));
    }
    Ok(())
}
