use std::env;
extern crate hyper;
extern crate regex;
#[macro_use] extern crate lazy_static;

use std::net::ToSocketAddrs;
use std::io;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::os::unix::io::IntoRawFd;
use hyper::{Get, Post};
use hyper::uri::RequestUri;
use hyper::status::StatusCode;
use hyper::server::{Server, Handler, Request, Response};

use regex::Regex;

static ENV_PORT:&'static str = "ROCK_STORAGE_PORT";
static ENV_ROOT_DIR:&'static str = "ROCK_STORAGE_DIR";

struct SenderHandler {
  root_dir: String,
}

impl SenderHandler {
  fn try_serve_file(&self, req: &Request, mut responce: Response, path: &String) {
    let full_path = format!("{}/{}", self.root_dir, path);
    println!("serving file {} as {}", path, full_path);
    match File::open(full_path) {
      Ok(mut file) => {
        let mut res = responce.start().unwrap();
        let mut buff = [0; 102400];
        loop {
          match file.read(&mut buff) {
            Ok(0) => break, //eof
            Ok(size) => res.write_all(&buff[0 .. size]).unwrap(),
            Err(e) => {
              println!("error reading {} : {}", path, e);
              break;
            }
          }
        }
        res.flush();
      },
      Err(error) => {
        let message = format!("error for {}: {}\n", path, error);
        let buff = message.into_bytes();
        *responce.status_mut() = StatusCode::NotFound;
        responce.send(&buff).unwrap();
      }
    }
  }

  fn try_del_file(&self, req: &Request, mut responce: Response, file_path: &str) {
    let full_path = format!("{}/{}", self.root_dir, file_path);
    println!("deleting {} => {}", file_path, full_path);
    match fs::remove_file(full_path) {
      Ok(v) =>  { responce.start().unwrap().flush(); },
      Err(error) => {
        let message = format!("error deleting {}: {}\n", file_path, error);
        let buff = message.into_bytes();
        *responce.status_mut() = StatusCode::Forbidden;
        responce.send(&buff).unwrap();
      }
    }
  }
}

impl Handler for SenderHandler {
  fn handle(&self, req: Request, res: Response) {
    if req.method == Get {
      if let RequestUri::AbsolutePath(ref path) = req.uri  {
        lazy_static! {
          static ref DEL_RE: Regex = Regex::new("/delete[?]file=(.+)").unwrap();
        }
        println!("path: {}", path);
        match DEL_RE.captures(path) {
          Some(captures) => {
            let file_path = captures.at(1).unwrap();
            self.try_del_file(&req, res, file_path);
          },
          None => self.try_serve_file(&req, res, path),
        }
      }
    }
  }
}


fn main() {
  let mut port:u32 = 1234;

  if let Ok(value) = env::var(ENV_PORT) {
    match value.parse() {
      Ok(port_value) => { port = port_value },
      Err(error) => {
        println!("The value {} cannot be used as port : {}", value, error);
      }
    }
  }

  let root_dir = match env::var(ENV_ROOT_DIR) {
    Ok(value) => { value },
    _ => { ".".to_string() }
  };

  let address = format!("0.0.0.0:{}", port);

  println!("Starting at port {}, root_dir = {}", port, root_dir);

  //let server = Server::http(&"127.0.0.1:1337".parse().unwrap()).unwrap();
  //let server = Server::http(&"127.0.0.1:1337".parse().unwrap()).unwrap();

  //let server:Server<HttpListener> = Server::http(&address).unwrap();
  let server = Server::http("0.0.0.0:1234").unwrap().handle(SenderHandler {
    root_dir: root_dir
  }).unwrap();
}
