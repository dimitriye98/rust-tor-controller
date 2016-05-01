#[macro_use]
extern crate log;
extern crate env_logger;
extern crate regex;

//use std::path::Path;
use std::net::{SocketAddr, TcpStream, Shutdown};
use std::io::{Read, Write};
//use std::str;
use std::io::{BufReader, BufRead, BufWriter};
//use std::option::Option;

use regex::Regex;

//enum Auth {
//    None,
//    Cookie(Path),
//    HashPass(&str),
//}

//enum Connection {
//    Tcp(SocketAddr),
//    Unix(Path),
//}

struct Connection<T: Read + Write> {
    raw_stream: T,
    buf_reader: BufReader<T>,
    buf_writer: BufWriter<T>,
}

struct Controller<T: Read + Write > {
    con: Connection<T>,
//    auth: Auth,
//    connection: Connection,
//    hash_pass: Option<&str>,
}

impl Controller<TcpStream> {
    fn from_port(port: u16) -> Controller<TcpStream> {
        let raw_stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let buf_reader = BufReader::new(raw_stream.try_clone().unwrap());
        let buf_writer = BufWriter::new(raw_stream.try_clone().unwrap());
        Controller {
            con: Connection {
                raw_stream: raw_stream,
                buf_reader: buf_reader,
                buf_writer: buf_writer,
            }
        }
    }
}

impl<T: Read + Write> Controller<T> {
    fn authenticate(&mut self) {
        unimplemented!();
    }
    fn raw_cmd(&mut self, cmd: &str) -> Vec<String> {
        debug!("{}", cmd);
        self.con.buf_writer.write_all(cmd.as_bytes()).unwrap();
        self.con.buf_writer.write_all("\r\n".as_bytes()).unwrap();
        self.con.buf_writer.flush().unwrap();
        let mut raw_line = String::new();
        let mut reply = Vec::new();

        self.con.buf_reader.read_line(&mut raw_line).unwrap();
        {
            let line = &raw_line[..raw_line.len()-2];
            debug!("{}", line);
            reply.push(line.to_string());
        }
        let status_code = &raw_line.clone()[..3];
        raw_line.clear();

        while self.con.buf_reader.read_line(&mut raw_line).unwrap() > 0 {
            {
                let line = &raw_line[..raw_line.len()-2];
                debug!("{}", line);
                reply.push(line.to_string());
                if &line[..3] != status_code {
                    panic!("Reply Error");
                }
                if &line[3..4] == " " {
                    break;
                }
            }
            raw_line.clear();
        }
        reply
    }
    fn protocolinfo(&mut self) {
        let reply = self.raw_cmd("PROTOCOLINFO");
        let re_protocolinfo = 
            Regex::new("^250-PROTOCOLINFO (?P<version>[0-9]+)$")
            .unwrap();
        let re_tor_version = 
            Regex::new("^250-VERSION Tor=\"(?P<tor_version>[^\"]*)\"(?P<opt_arguments>.*)$")
            .unwrap();
        let re_auth =
            Regex::new("^250-AUTH METHODS=(?P<auth_methods>[A-Z,]+)[  ]*(?P<maybe_cookie_files>.*)$")
            .unwrap();
        let re_cookiefile =
            Regex::new("COOKIEFILE=\"(?P<cookie_file>[^\"]*)\"")
            .unwrap();
        let prot_inf = re_protocolinfo.captures(reply[0].as_str()).unwrap();
        let version = prot_inf.name("version").unwrap();
        for line in &reply[1..] {
            match line.split(" ").collect::<Vec<_>>()[0] {
                "250-AUTH" => {
                    let auth = re_auth.captures(line).unwrap();
                    let auth_methods = auth.name("auth_methods").unwrap();
                    let maybe_cookie_files = auth.name("maybe_cookie_files").unwrap();
                    for caps in re_cookiefile.captures_iter(maybe_cookie_files) {
                        debug!("Cookie file={}", caps.name("cookie_file").unwrap());
                    }
                    debug!("Auth methods={}", auth_methods);
                },
                "250-VERSION" => {
                    let ver = re_tor_version.captures(line).unwrap();
                    let tor_version = ver.name("tor_version").unwrap();
                    let opt_arguments = ver.name("opt_arguments").unwrap();
                    debug!("Tor version={}, optional args={}", tor_version, opt_arguments);
                },
                "250" => debug!("OK"),
                _ => (),
            } 
        }
        debug!("version = {}", version);
    }
//    fn connect(mut &self) {
//        match self.connection {
//            Tcp(addr) => self.stream = TcpStream::connect(addr).unwrap(),
//            Unix(path) => unimplemented!(),
//        }
//    }
//    fn close(mut &self) {
//        self.stream.shutdown(Shutdown::Both).unwrap();
//    }
}

fn main() {
    env_logger::init().unwrap();

    info!("Starting Tor Controller!");
    let mut controller = Controller::from_port(9051);
    //controller.assert("PROTOCOLINFO");
    controller.protocolinfo();
//    controller.write("PROTOCOLINFO\r\n");
//    controller.write("PROTOCOLINFO\r\n");
}
