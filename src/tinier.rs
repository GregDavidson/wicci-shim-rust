// Wicci Shim Module
// Http Request/Response Interface
// Wrapper for tiny_http

use std::io::{Read,Cursor};
use tiny_http;
use std::str::FromStr;
use std::process;
use html;
use std::fmt::Write;

lazy_static! {
  pub static ref GET: tiny_http::Method = tiny_http::Method::from_str("GET").unwrap();
  pub static ref PUT: tiny_http::Method = tiny_http::Method::from_str("PUT").unwrap();
}

pub fn open_server(port: u16) -> tiny_http::Server {
  tiny_http::ServerBuilder::new().
		with_port(port).build().unwrap_or_else( | err | {
      // log failure!! shutdown server gracefully!!
      errorln!("open_server fails with {:?}", err);
      process::exit(10);
		})
}

pub fn cursor_on<D>(data: D)->Cursor<Vec<u8>> where D: Into<Vec<u8>> {
  Cursor::new(data.into())
}

pub fn str_response(
  status_code: i32, headers: Vec<tiny_http::Header>, str_data: String
    ) -> tiny_http::Response<Cursor<Vec<u8>>> {
  let data_len = str_data.len();
  tiny_http::Response::new(
    tiny_http::StatusCode::from(status_code), headers,
    cursor_on(str_data), Some(data_len), None )
}

pub fn hdr_response(
  status_code: i32, headers: Vec<tiny_http::Header>
    )-> tiny_http::Response<Cursor<Vec<u8>>> {
  tiny_http::Response::new(
    tiny_http::StatusCode::from(status_code), headers,
    cursor_on(Vec::with_capacity(0)), Some(0), None )
}

// pub fn append_request below
// Re-Encodes tiny_http::Request as a byte vector

// error: missing lifetime specifier [E0106]
// type Buf = &mut Option<&mut Vec<u8>>;

fn append_bytes(maybe_buf: &mut Option<&mut Vec<u8>>, bytes: &[u8]) -> usize {
  match maybe_buf.as_mut() {
    None => { },
    Some(buf) => {
      buf.extend(bytes.iter().cloned());
    }
  }
  bytes.len()
}

fn append_bytes_delim(
  maybe_buf: &mut Option<&mut Vec<u8>>, bytes: &[u8], delim: &[u8]
) -> usize {
  append_bytes(maybe_buf, bytes) + append_bytes(maybe_buf, delim)
}

fn append_headers(
  maybe_buf: &mut Option<&mut Vec<u8>>, hdrs: &[tiny_http::Header]
) -> usize {
  let cs = b": ";
  let nl = b"\r\n";             // need '\r' for testing
  hdrs.iter().fold(0, |_i, h| {
    append_bytes_delim(maybe_buf, &h.field.as_str().as_bytes(), cs)
      + append_bytes_delim(maybe_buf, &h.value.as_bytes(), nl)
  })
}

fn append_body(
  maybe_buf: &mut Option<&mut Vec<u8>>, body: &mut Read, size: usize
 ) -> usize {
  match maybe_buf.as_mut() {
    None => size,
    Some(buf) => body.read_to_end(buf).unwrap()
      // handle failure possibility in unwrap()!!
  }
}

fn append_http_version(
  maybe_buf: &mut Option<&mut Vec<u8>>, v: &tiny_http::HTTPVersion
) -> usize {
  match maybe_buf.as_mut() {
    None => { },
    Some(b) => {
      let (major, minor) = (v.0, v.1);
      assert!(major < 10);
      assert!(minor < 10);
      b.push(b'0' + major);
      b.push(b'.');
      b.push(b'0' + minor);
      b.push(b' ');
    }
  }
  4
}

fn append_request_sans_body(
  b: &mut Option<&mut Vec<u8>>, r: &tiny_http::Request
) -> usize {
  let sp = b" ";
  let nl = b"\r\n";             // need '\r' for testing
  append_bytes_delim(b, &r.method().as_str().as_bytes(), sp)
    + append_bytes_delim(b, &r.url().as_bytes(), sp)
    + append_http_version(b, &r.http_version())
    + append_headers(b, r.headers())
    + append_bytes(b, nl)
}

pub fn append_request(
  b: &mut Option<&mut Vec<u8>>, r: &mut tiny_http::Request
) -> usize {
  let len_sans_body = append_request_sans_body(b, r);
  let body_len = r.body_length().unwrap_or(0);
  let mut body_reader = r.as_reader();
  len_sans_body + append_body(b, body_reader, body_len)
}

pub fn echo_requests(server: tiny_http::Server) {
	for r in server.incoming_requests() {
    if r.url() == "/favicon.ico" {
      r.respond( hdr_response(404, Vec::with_capacity(0)) );
      continue;
    }
    println!("method: {}", r.method());
    println!("url: {}", r.url());
    println!("http_version: {}", r.http_version());
    for h in r.headers().iter() {
      println!("{}: {}", h.field, h.value);
    }
    if r.method().eq(&*GET) || r.method().eq(&*PUT) {
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
      r.respond(str_response(200, headers, html_str));
    }
    println!("");
  }
}


#[cfg(test)]
mod test {
  // use super::*;
  
  #[test]
  fn test1() {
    
  }
}

// Notes for future improvements:

/*
let s = [0u8, 1u8, 2u8];
let mut v = Vec::new();
v.extend(s.iter().map(|&i| i)); // requires a closure on every value
v.extend(s.to_vec().into_iter()); // allocates an extra copy of the slice

v.extend(s.iter().cloned());

That is effectively equivalent to using .map(|&i| i) and it does minimal copying.

v + &s will work on beta, which I believe is just similar to pushing each value onto the original Vec.

*/
