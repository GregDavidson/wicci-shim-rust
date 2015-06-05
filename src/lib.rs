#![feature(plugin)]
#![plugin(regex_macros)]
// #![feature(exit_status)]  // set_exit_status unstable as of 1.1
// #[macro_use]
// extern crate regex_macros;
extern crate regex;
extern crate tiny_http;
// extern crate ascii;
extern crate libc;
use tinier::*;
use html::*;
mod tinier;
mod html;

// use std::fmt::{self, Write};
use std::fmt::Write;

// use libc::funcs::c95::ctype;
// use std::io::{self, Write};
// use std::io::{BufReader,Read,Cursor};
use std::str::{FromStr};

// use tiny_http::{Server};

// define database connection pool structures

// Hyper manages workers!
// tiny-http says it does too!

pub fn echo_requests(server: tiny_http::Server) {
  let get_ = tiny_http::Method::from_str("GET").unwrap();
  let put_ = tiny_http::Method::from_str("PUT").unwrap();
	for r in server.incoming_requests() {
    if r.get_url() == "/favicon.ico" {
      r.respond( hdr_response(404, Vec::with_capacity(0)) );
      continue;
    }
    println!("method: {}", r.get_method());
    println!("url: {}", r.get_url());
    println!("http_version: {}", r.get_http_version());
    for h in r.get_headers().iter() {
      println!("{}: {}", h.field, h.value);
    }
    if r.get_method().eq(&get_) || r.get_method().eq(&put_) {
      let mut header_data = String::new();
      for h in r.get_headers().iter() {
        writeln!(&mut header_data,
                 "<dt>{}</dt>\n<dd>{}</dd>",
                 h.field, h.value );
      }
      let headers: Vec<tiny_http::Header> = Vec::new();
      let html_str = html_title_contents("shim response",
        html_tag_contents(
          "dl", vec!(
            html_tag_content("dt", html_static("method")),
            html_tag_content("dd", format!("{}", r.get_method())),
            html_tag_content("dt", html_static("url")),
            html_tag_content("dd", format!("{}", r.get_url())),
            html_tag_content("dt", html_static("http_version")),
            html_tag_content("dd", format!("{}", r.get_http_version())),
            header_data
          )
        )
      );
      r.respond(str_response(200, headers, html_str));
    }
    println!("");
  }
}
