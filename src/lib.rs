#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;
extern crate tiny_http;
use tinier::*;
use html::*;
pub mod tinier;
pub mod html;

use std::fmt::Write;
use std::str::FromStr;

// define database connection pool structures

// Hyper manages workers!
// tiny-http says it does too!

pub fn echo_requests(server: tiny_http::Server) {
  let get_ = tiny_http::Method::from_str("GET").unwrap();
  let put_ = tiny_http::Method::from_str("PUT").unwrap();
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
    if r.method().eq(&get_) || r.method().eq(&put_) {
      let mut header_data = String::new();
      for h in r.headers().iter() {
        writeln!(&mut header_data,
                 "<dt>{}</dt>\n<dd>{}</dd>",
                 h.field, h.value );
      }
      let headers: Vec<tiny_http::Header> = Vec::new();
      let html_str = html_title_contents("shim response",
        html_tag_contents(
          "dl", vec!(
            html_tag_content("dt", html_static("method")),
            html_tag_content("dd", format!("{}", r.method())),
            html_tag_content("dt", html_static("url")),
            html_tag_content("dd", format!("{}", r.url())),
            html_tag_content("dt", html_static("http_version")),
            html_tag_content("dd", format!("{}", r.http_version())),
            header_data
          )
        )
      );
      r.respond(str_response(200, headers, html_str));
    }
    println!("");
  }
}
