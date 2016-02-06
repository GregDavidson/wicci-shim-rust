// Wicci Shim Program

// #![feature(plugin)]
// #![plugin(regex_macros)]

// #![feature(convert)]

#[macro_use] extern crate log;
extern crate env_logger;
// use log::LogLevel;

// extern crate hyper;				// more than we need?
extern crate tiny_http;       // more modest
extern crate regex;

extern crate ascii;
// use ascii::AsciiStr;
use std::ascii::AsciiExt;

extern crate postgres;
use postgres::stmt::Statement;

#[macro_use]
extern crate lazy_static;

use std::process;
use std::fmt::Write;
use std::sync::Arc;
use std::thread;

extern crate getopts;

pub mod options;
pub mod tests;
pub mod tinier;
pub mod html;
pub mod db;

fn echo_requests(server: tiny_http::Server) {
	for r in server.incoming_requests() {
    if r.url() == "/favicon.ico" {
      r.respond( tinier::hdr_response(404, Vec::with_capacity(0)) ).unwrap_or_else( |err| {
        error!("favicon 404 response error {}", err);
     } );
      continue;
    }
    println!("method: {}", r.method());
    println!("url: {}", r.url());
    println!("http_version: {}", r.http_version());
    for h in r.headers().iter() {
      println!("{}: {}", h.field, h.value);
    }
    if r.method().eq(&*tinier::GET) || r.method().eq(&*tinier::PUT) {
      let mut header_data = String::new();
      for h in r.headers().iter() {
        writeln!(&mut header_data,
                 "<dt>{}</dt>\n<dd>{}</dd>",
                 h.field, h.value ).unwrap();
        // and if unwrap() fails??
      }
      let headers: Vec<tiny_http::Header> = Vec::new();
      let html_str = html::html_title_contents("shim response",
        html::html_tag_contents(
          "dl", vec!(
            html::html_tag_content("dt", html::html_static("method")),
            html::html_tag_content("dd", format!("{}", r.method())),
            html::html_tag_content("dt", html::html_static("url")),
            html::html_tag_content("dd", format!("{}", r.url())),
            html::html_tag_content("dt", html::html_static("http_version")),
            html::html_tag_content("dd", format!("{}", r.http_version())),
            header_data
          )
        )
      );
      r.respond(tinier::str_response(200, headers, html_str)).unwrap_or_else( |err| {
        error!("200 response error {}", err);
      } );
    }
    println!("");
  }
}

#[cfg(feature = "never")]
fn make_lower<T: AsciiExt>(bytes: &mut T) {
  bytes.make_ascii_lowercase();
}

fn make_lower_vec_u8(bytes: &mut Vec<u8>) {
  for i in 0 .. bytes.len() {
    bytes[i] = bytes[i].to_ascii_lowercase();
  }
}

// tried to make this generic over numeric types but
// in Rust 1.0 ... 1.1 this is now hard!
fn digits_to_usize(digits: &Vec<u8>) -> Option<usize> {
  let mut val: usize = 0;
  for d in digits {
    if *d < b'0' || *d > b'9' { return None; }
    val = val * 10 + (*d - b'0') as usize;
  }
  Some(val)
}

fn stmt_req_rows<'a>(
  stmt: &'a Statement, req: &'a mut tiny_http::Request
)-> postgres::rows::Rows<'a> {
  let mut no_vec:  Option<&mut Vec<u8>> = None;
  let req_len = tinier::append_request(&mut no_vec, req);
  let mut req_buf = Vec::<u8>::with_capacity(req_len);
  let req_len_ = tinier::append_request(&mut Some(&mut req_buf), req);
  assert!(req_len == req_len_);
  stmt.query(&[&req_buf]).unwrap_or_else( | err | { // too few args!!!
    error!("two few query args {}", err);
    // send client sad Reponse structure!!
    // continue to next requset!!
    let buf = String::from_utf8(req_buf).unwrap_or(String::from("???"));
    error!("stmt_req_rows db query error {:?} on {}", err, &buf);
    process::exit(1);
  })
}

#[derive(Debug)]
struct ResponseData {
  headers: Vec<tiny_http::Header>,
  status_code: i32,
  content_length: usize,
  body: Vec<u8>
}

fn stmt_req_response(stmt: &Statement, req: &mut tiny_http::Request) -> ResponseData {
  let rows = stmt_req_rows(stmt, req);
  if *options::DBUG { println!("Request {:?}", rows); }
  let mut rd = ResponseData {
    status_code: -1, // any better default?
    headers: Vec::<tiny_http::Header>::with_capacity( rows.iter().count() ),
    body: Vec::<u8>::with_capacity(0),
    content_length: 0
  };
  for row in &rows {
    let mut hdr_bytes: Vec<u8> = row.get(0);
    make_lower_vec_u8(&mut hdr_bytes);    
    let text_bytes:Vec<u8> = row.get(1);
    let bin_bytes:Vec<u8> = row.get(2);
    match hdr_bytes.as_slice() {
      b"_status" => match digits_to_usize(&text_bytes) {
        Some(code) => rd.status_code = code as i32,
        None => {
          // send appropriate error response to client!!
          // log failure!!
          // continue to next request!!
          error!("illegal db header value {:?}: {:?}", hdr_bytes, text_bytes);
          process::exit(2);
        }
      },
      b"content-length" => match digits_to_usize(&text_bytes) {
        Some(length) => rd.content_length = length,
        None => {
          // send appropriate error response to client!!
          // log failure!!
          // continue to next request!!
          error!("illegal db header value {:?}: {:?}", hdr_bytes, text_bytes);
          process::exit(3);
        }
      },
      b"_body_bin" => { rd.body = bin_bytes; },
      b"_body" => { rd.body = text_bytes; },
      _ => {
        assert!(text_bytes.len() != 0);
        rd.headers.push(tiny_http::Header::from_bytes(hdr_bytes, text_bytes).unwrap());
        // handle failure possibility in unwrap()!!
      }
    }
  }
  // make sure we have a valid status_code, content_length, etc.
  assert!(rd.content_length == rd.body.len());
  assert!(rd.status_code > 0);
  if *options::DBUG { println!("Response {:?}", rd);  }
  rd
}

fn stmt_req_respond(stmt: &Statement, mut req: tiny_http::Request) {
  let rd = stmt_req_response(stmt, &mut req);
  req.respond( tiny_http::Response::new(
    tiny_http::StatusCode::from(rd.status_code),
    rd.headers, tinier::cursor_on(rd.body), Some(rd.content_length), None ) ).unwrap_or_else( |err| {
      error!("stmt_req_respond error {}", err);
    } );
}

fn handle_requests(server: Arc<tiny_http::Server>) {
  let mut guards = Vec::with_capacity(*options::NUM_WORKERS);
  
  for _ in 0 .. *options::NUM_WORKERS {
    let server = server.clone();
    let guard = thread::spawn(move || {
      let db = db::connect();
      let query_stmt = db::prepare(&db);
      loop {
        let request = server.recv().unwrap();
        // handle failure possibility in unwrap()!!
        stmt_req_respond(&query_stmt, request);
      }
    });
    
    guards.push(guard);
  }
  
  for g in guards { g.join().unwrap(); } // why unwrap??
}

fn main() {
  env_logger::init().unwrap();
  if options::opt_present("help") { options::print_usage(); return; }
  if options::opt_present("test") { tests::do_tests(); return; }
	let server = tinier::open_server(*options::HTTP_PORT);
  if options::opt_present("echo") {
    echo_requests(server);
    return;
  }
  handle_requests(Arc::new(server));
}
