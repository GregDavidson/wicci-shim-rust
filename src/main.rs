// Wicci Shim Program

#![feature(plugin)]
// #![plugin(regex_macros)]

#![feature(convert)]

extern crate ascii;             // for text module
extern crate encoding;
extern crate getopts;
extern crate regex;
extern crate tiny_http;       // more modest than hyper

extern crate postgres;
use postgres::stmt::Statement;

#[macro_use]
extern crate lazy_static;

use std::process;
// use std::fmt::Write;
use std::sync::Arc;
use std::thread;

pub mod macros;
pub mod options;
pub mod tests;
pub mod tinier;
pub mod text;
pub mod html;
pub mod db;

fn stmt_req_rows<'a>(
  stmt: &'a Statement, req: &'a mut tiny_http::Request
)-> postgres::rows::Rows<'a> {
  let mut no_vec:  Option<&mut Vec<u8>> = None;
  let req_len = tinier::append_request(&mut no_vec, req);
  let mut req_buf = Vec::<u8>::with_capacity(req_len);
  let req_len_ = tinier::append_request(&mut Some(&mut req_buf), req);
  assert!(req_len == req_len_);
  stmt.query(&[&req_buf]).unwrap_or_else( | err | {
    // log error, send client sad Reponse structure, continue to next request!!
    let buf = String::from_utf8(req_buf).unwrap_or(String::from("???"));
    errorln!("stmt_req_rows db query error {:?} on {}", err, &buf);
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

fn stmt_req_response(stmt: &Statement, req: &mut tiny_http::Request)-> ResponseData {
  let rows = stmt_req_rows(stmt, req);
  if *options::DBUG { println!("Request {:?}", rows); }
  let mut rd = ResponseData {
    status_code: -1, // any better default?
    headers: Vec::<tiny_http::Header>::with_capacity( rows.iter().count() ),
    body: Vec::<u8>::with_capacity(0),
    content_length: 0
  };
  for row in rows {
    let mut hdr_bytes: Vec<u8> = row.get(0);
    text::make_lower_vec_u8(&mut hdr_bytes);    
    let text_bytes:Vec<u8> = row.get(1);
    let bin_bytes:Vec<u8> = row.get(2);
    match hdr_bytes.as_slice() {
      b"_status" => match text::digits_to_usize(&text_bytes) {
        Some(code) => rd.status_code = code as i32,
        None => {
          // send appropriate error response to client!!
          // log failure!!
          // continue to next request!!
          errorln!("illegal db header value {:?}: {:?}", hdr_bytes, text_bytes);
          process::exit(2);
        }
      },
      b"content-length" => match text::digits_to_usize(&text_bytes) {
        Some(length) => rd.content_length = length,
        None => {
          // send appropriate error response to client!!
          // log failure!!
          // continue to next request!!
          errorln!("illegal db header value {:?}: {:?}", hdr_bytes, text_bytes);
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
    rd.headers, tinier::cursor_on(rd.body), Some(rd.content_length), None ) );
}

fn handle_requests(server: Arc<tiny_http::Server>) {
  let mut guards = Vec::with_capacity(*options::NUM_WORKERS);
  
  for _ in 0 .. *options::NUM_WORKERS {
    let server = server.clone();
    let guard = thread::spawn(move || {
      let db = db::connect();
      let query_stmt = db::prepare_wicci_serve(&db);
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
  if options::opt_present("help") { options::print_usage(); return; }
  if options::opt_present("test") { tests::do_tests(); return; }
	let server = tinier::open_server(*options::HTTP_PORT);
  if options::opt_present("echo") { tinier::echo_requests(server); return; }
  handle_requests(Arc::new(server));
}


#[cfg(test)]
mod test {
//  use super::*;
  
  #[test]
  fn test1() {
    
  }
}
