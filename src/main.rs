#![feature(plugin)]
// #![plugin(regex_macros)]
#![feature(exit_status)]  // set_exit_status unstable as of 1.1
#[macro_use]
extern crate lazy_static;
extern crate getopts;
// extern crate hyper; // more than we need?
extern crate tiny_http;
// extern crate libc;
// extern crate regex_macros;
// extern crate regex;

extern crate ascii;
use ascii::AsciiString;

extern crate postgres;
use postgres::{Connection, Statement, SslMode};
use postgres::error::ConnectError;
use postgres::Result as PG_Result;


extern crate shim;
use shim::*;
use std::sync::Arc;
use std::thread;
use std::str::FromStr;

use std::io::Read;

// \/ option and argument management \/

const NUM_WORKERS_DEFAULT: usize = 4;
const HTTP_PORT_DEFAULT: u16 = 8080;
const DB_PORT_DEFAULT: u16 = 5432;
const DB_HOST_DEFAULT: &'static str = "localhost";
const DB_NAME_DEFAULT: &'static str = "wicci1";
const DB_INIT_FUNC_DEFAULT: &'static str = "wicci_ready()";
const DB_FUNC_DEFAULT: &'static str = "wicci_serve";
// --> DB_USER_DEFAULT defined below!

const ASCII_LF: u8 = 10;
const ASCII_CR: u8 = 13;
const ASCII_SP: u8 = 32;
const ASCII_DOT: u8 = 46;
const ASCII_0: u8 = 48;
const ASCII_COLON: u8 = 58;

enum ReqHdr {
  Status,
  ContentLength,
  Body,
  BodyBin
}

use std::collections::BTreeMap;

type ReqHdrBytes = (BTreeMap<ReqHdr, [u8]>, BTreeMap<[u8], ReqHdr>);

fn req_hdr_bytes( rhb: & mut ReqHdrBytes, rh: ReqHdr, bytes: &'static[u8] ) {
  rhb.0.insert(rh, bytes);
  rhb.1.insert(bytes, h);
}

fn req_hdr_bytes( rhb: & mut ReqHdrBytes, rh: ReqHdr, bytes: &'static str ) {
  req_hdr_bytes( rhb, rh, str.
}

// fetch options which have a default

/* I would strongly prefer to not be using String
   and/or any other heap-allocated types.  Failing
   that, I'm going to try to make any such objects
   less ephemeral by, e.g. making them lazy static.
   Failing that, I'll try to make them very ephemeral
   so that it would be easier to stack-allocate them
   in the future!
 */

type OptStr = String;

fn opt_default<T: FromStr>(opt_name: &str, dfalt: T)->T {
  match PGM_OPTS.opt_str(opt_name) {
    None => dfalt,
    Some(p) => T::from_str(&p).unwrap_or_else( |err| {
      std::env::set_exit_status(10);
      // no method named `to_string` found for type `<T as core::str::FromStr>::Err`
      // panic!(err.to_string());
      panic!(p);
    } )
  }
}

fn opt_str_default(opt_name: &str, dfalt: &str)->String {
  opt_default::<String>("init-func", dfalt.to_string())
}

lazy_static! {
  static ref PGM_ARGV: Vec<String> = {
    let argv = std::env::args().collect();
    argv
  };
  static ref PGM_NAME: String = PGM_ARGV[0].clone();
  static ref PGM_ARGS: &'static[String] = &PGM_ARGV[1..];
  
  static ref PGM_OPTIONS: getopts::Options = {
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("d", "debug", "trace what's going on");
//    opts.optflag("s", "show-args", "show program arguments");
//	  opts.optflag("b", "debug-save-blobs", "save received blobs to files");
    opts.optflag("e", "echo", "echo requests readably");
	  opts.optopt("w", "num-workers", "", ""); // NUM_WORKERS_DEFAULT
	  opts.optopt("p", "http-port", "", ""); // HTTP_PORT_DEFAULT
	  opts.optopt("I", "db-init-func", "", ""); // DB_INIT_FUNC_DEFAULT
	  opts.optopt("F", "db-func", "", ""); // DB_FUNC_DEFAULT
	  // db connection attributes: see DBOption
	  opts.optopt("H", "db-host", "", "db server port"); // DB_HOST_DEFAULT
    // opts.optopt("A", "db-host-addr", "", ""); // why not just allow numeric db-host??
    opts.optopt("P", "db-port", "", ""); // DB_PORT_DEFAULT
	  opts.optopt("N", "db-name", "", ""); // DB_NAME_DEFAULT
    opts.optopt("U", "db-user", "", "");
	  opts.optopt("", "db-password", "", "");
//	  opts.optopt("", "db-connect-timeout", "", "");
    opts
  };
  
  static ref PGM_OPTS: getopts::Matches
		= PGM_OPTIONS.parse( PGM_ARGS.iter() ).
		unwrap_or_else( |err| {
      std::env::set_exit_status(11);
      panic!(err.to_string());
   	} );

  static ref DB_USER_DEFAULT: &'static str = "greg"; // get current user???

  static ref DBUG: bool = PGM_OPTS.opt_present("debug");

  static ref NUM_WORKERS: usize =
    opt_default::<usize>("num-workers", NUM_WORKERS_DEFAULT);
  static ref HTTP_PORT: u16 = opt_default::<u16>("http-port", HTTP_PORT_DEFAULT);
  static ref DB_INIT_FUNC: OptStr =
    opt_str_default("db-init-func", DB_INIT_FUNC_DEFAULT);
  static ref DB_INIT_STR: String =
    format!("SELECT {}('{}')", *DB_INIT_FUNC, *PGM_NAME); /*
  Need to sql_literal(DB_INIT_FUNC) and sql_quote(PGM_NAME)!!!
   */
  static ref DB_FUNC: OptStr = opt_str_default("db-func", DB_FUNC_DEFAULT);
  static ref DB_QUERY_STR: String =
    format!("SELECT h,v,b FROM {}($1, '_body_bin') AS foo(h,v,b)",
            *DB_FUNC);   //  Need to sql_literal(DB_FUNC)!!!
  static ref DB_HOST: OptStr = opt_str_default("db-host", DB_HOST_DEFAULT);
  static ref DB_PORT: u16 = opt_default::<u16>("db-port", DB_PORT_DEFAULT);
  static ref DB_USER: OptStr = opt_str_default("db-user", *DB_USER_DEFAULT);
  static ref DB_NAME: OptStr = opt_str_default("db-name", DB_NAME_DEFAULT);

  // e.g. "postgresql://greg@localhost/greg";
  static ref PG_DSN: String = {
    let pw = PGM_OPTS.opt_present("db-password");
    format!(
      "postgresql://{}{}{}@{}/{}", *DB_USER,
      if pw { ":" } else { "" },
      if pw { PGM_OPTS.opt_str("db-password").unwrap() } else { "".to_string() },
      *DB_HOST, *DB_NAME )
  };

}                              // lazy_static!

// other command-line management

fn print_usage() {
    let brief = format!("Usage: {} [options]...", *PGM_NAME);
    print!("{}", PGM_OPTIONS.usage(&brief));
}

// /\ option and argument management /\

// fn foo(db: &Connection) {
//   let f:() = Connection::connect("", &SslMode::None);
//   let g:() = db.prepare("");
//	core::result::Result<postgres::Statement<'_>, postgres::error::Error>
// }

fn pg_try_connect(dsn: &str) -> Result<Connection, ConnectError> {
    let conn = match Connection::connect(dsn, &SslMode::None) {
        Ok(conn) => {
            if *DBUG { println!("Connected to: {}", dsn) };
            Ok(conn)
        },
        Err(e) => {
            if *DBUG { println!("Connection error: {}", e) };
            Err(e)
        }
    };
    conn
}

fn pg_connect() -> Connection {
  pg_try_connect(&*PG_DSN).unwrap_or_else( | err | {
    std::env::set_exit_status(12);
    panic!(err.to_string());
	})
}

fn pg_try_prepare<'a>(db: &'a Connection, sql_str: &str) -> PG_Result<Statement<'a>> {
  let maybe_sql = db.prepare(sql_str);
  match maybe_sql {
    Ok(_) => (),
    Err(e) => if *DBUG {
      println!("Preparing query {} failed with {}", sql_str, e)
    } else { () }
  };
  maybe_sql
}

fn pg_prepare(db: & Connection) -> Statement {
  let stmt = pg_try_prepare(db, &*DB_QUERY_STR).unwrap_or_else( | err | {
    std::env::set_exit_status(13);
    panic!(err.to_string());
  });
  stmt
}

/*
let s = [0u8, 1u8, 2u8];
let mut v = Vec::new();
v.extend(s.iter().map(|&i| i)); // requires a closure on every value
v.extend(s.to_vec().into_iter()); // allocates an extra copy of the slice

v.extend(s.iter().cloned());

That is effectively equivalent to using .map(|&i| i) and it does minimal copying.

v + &s will work on beta, which I believe is just similar to pushing each value onto the original Vec.

*/

fn append_1(maybe_buf: Option<& mut Vec<u8>>, bytes: &[u8]) -> usize {
  match maybe_buf {
    None => { },
    Some(buf) => {
      buf.extend(bytes.iter().cloned());
    }
  }
  bytes.len()
}

fn append_2(maybe_buf: Option<& mut Vec<u8>>, bytes: &[u8], delim: &[u8]) -> usize {
  append_1(maybe_buf, bytes) + append_1(maybe_buf, delim)
}

fn append_headers(maybe_buf: Option<& mut Vec<u8>>, hdrs: &[tiny_http::Header]) -> usize {
  let cs = [ASCII_COLON, ASCII_SP];
  let nl = [ASCII_CR, ASCII_LF]; // need '\r' for testing
  hdrs.iter().fold(0, |i, h| {
    append_2(maybe_buf, &h.field.0.as_bytes(), &cs)
      + append_2(maybe_buf, &h.value.as_bytes(), &nl)
  })
}

fn append_body(maybe_buf: Option<& mut Vec<u8>>, body: &mut Read, size: usize) -> usize {
  match maybe_buf {
    None => size,
    Some(buf) => body.read_to_end(buf).unwrap()
  }
}

fn append_http_version(maybe_buf: Option<& mut Vec<u8>>, v: &tiny_http::HTTPVersion)
-> usize {
  match maybe_buf {
    None => { },
    Some(b) => {
      let (major, minor) = (v.0, v.1);
      assert!(major < 10);
      assert!(minor < 10);
      b.push(ASCII_0 + major);
      b.push(ASCII_DOT);
      b.push(ASCII_0 + minor);
      b.push(ASCII_SP);
    }
  }
  4
}

fn append_request(b: Option<& mut Vec<u8>>, r: & mut tiny_http::Request) -> usize {
  let sp = [ASCII_SP];
  let nl = [ASCII_CR, ASCII_LF]; // need '\r' for testing
  append_2(b, &r.method().0.as_bytes(), &sp)
    + append_2(b, &r.url().as_bytes(), &sp)
    + append_http_version(b, &r.http_version())
    + append_headers(b, r.headers())
    + append_1(b, &nl)
    + append_body(b, r.as_reader(), r.body_length().unwrap_or(0))
}

// or maybe just send it, rename to pg_respond??
fn pg_respond(stmt: &Statement, req: & mut tiny_http::Request) {
  let req_len = append_request(None, req);
  let mut req_buf = Vec::<u8>::with_capacity(req_len);
  let req_len_ = append_request(Some(& mut req_buf), req);
  assert!(req_len == req_len_);
  let rows = stmt.query(&[&req_buf]).unwrap_or_else( | err | {
    // replace with Reponse structure describing the error!!
    std::env::set_exit_status(14);
    panic!(err.to_string());
  });
  // let (at_least, _) = rows.size_hint(); // lazy_rows
  let mut status_code: i32 = -1; // any better default?
  let mut headers = Vec::<tiny_http::Header>::with_capacity( rows.iter().count() );
  let mut body: &Vec<u8>;
  let mut content_length: usize = 0; // any better default?
  for row in rows {
    let hdr_bytes = row.get(0) as Vec[u8];
    let text_bytes = row.get(1) as Vec[u8];
    let bin_bytes = row.get(2) as Vec[u8];
    match hdr_bytes {
      "_status" => match <i32>::parse(text_bytes) {
        Ok(code) => status_code = code,
        Err(err) => {         // how to just abort this request with error ???
          std::env::set_exit_status(25);
          panic!(err.to_string());
        }
      },
      "Content-Length" => match <usize>::parse(text_bytes) {
        Ok(length) => content_length = length,
        Err(err) => {         // how to just abort this request with error ???
          std::env::set_exit_status(15);
          panic!(err.to_string());
        }
      },
      "_body_bin" => { body = bin_bytes; },
      "_body" => { body = text_bytes; },
      _ => {
        assert!(text_bytes.len != 0);
        headers.push(tiny_http::Header::from_bytes(hdr_bytes, text_bytes));
      }
    }
  }
  // make sure we have a valid status_code, content_length, etc.
  assert!(content_length == body.length());
  assert!(status_code > 0);
  req.respond(tinier::str_response(status_code, headers, body));
}

fn handle_requests(server: Arc<tiny_http::Server>) {
  let mut guards = Vec::with_capacity(*NUM_WORKERS);

  for _ in (0 .. *NUM_WORKERS) {
    let server = server.clone();
    let guard = thread::spawn(move || {
      let db = pg_connect();
      let query_stmt = pg_prepare(db);
      loop {
        let mut request = server.recv().unwrap();
        let response = tiny_http::Response::from_string("hello world".to_string());
        pg_respond(&query_stmt, & mut request);
      }
    });
    
    guards.push(guard);
  }

  for g in guards { g.join().unwrap(); }
}

fn main() {
  if PGM_OPTS.opt_present("help") { print_usage(); }
	let port = *HTTP_PORT;
	let server = tiny_http::ServerBuilder::new().
		with_port(port).build().unwrap_or_else( | err | {
      std::env::set_exit_status(16);
      panic!(err.to_string());
		});
  if PGM_OPTS.opt_present("echo") {
    echo_requests(server);
    return;
  }
  handle_requests(Arc::new(server));
}
