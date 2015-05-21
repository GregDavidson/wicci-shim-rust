#![feature(plugin)]
#![plugin(regex_macros)]
#![feature(exit_status)]  // set_exit_status unstable as of 1.1
#[macro_use]
extern crate lazy_static;
extern crate getopts;
// extern crate hyper; // more than we need?
extern crate tiny_http;
extern crate ascii;
extern crate libc;
extern crate regex_macros;
extern crate regex;

use libc::funcs::c95::ctype;
use std::fmt::{self, Write};
// use std::io::{self, Write};
use std::io::{BufReader,Read};
use std::str::{FromStr};
use std::ascii::{AsciiExt};
use ascii::{AsciiStr, AsciiString};

//use core::format::Write;

lazy_static! {
    static ref PGM_ARGV: Vec<String> = {
        // let mut argv = Vec::new();
        // argv = std::env::args().collect();
        let argv = std::env::args().collect();
        argv
    };
    static ref PGM_NAME: String = PGM_ARGV[0].clone();
    static ref PGM_ARGS: &'static[String] = &PGM_ARGV[1..];
    
    static ref PGM_OPTIONS: getopts::Options = {
        let mut opts = getopts::Options::new();
        opts.optflag("h", "help", "print this help menu");
        opts.optflag("d", "debug", "trace what's going on");
        opts.optflag("D", "show-args", "show program arguments");
	      opts.optflag("B", "debug-save-blobs", "save received blobs to files");
	      opts.optopt("P", "http-port", "", ""); // dfalt: "8080";
	      opts.optopt("F", "init-func", "", ""); // dfalt: "wicci_ready";
	      // db connection attributes: see DBOption
	      opts.optopt("", "db-host", "", "db server port"); // dfalt: "localhost"
        opts.optopt("", "db-host-addr", "", "");
        opts.optopt("", "db-port", "", ""); // dfalt: "5432"
	      opts.optopt("", "db-name", "", ""); // "wicci1";
        opts.optopt("", "db-user", "", "");
	      opts.optopt("", "db-password", "", "");
	      opts.optopt("", "db-connect-timeout", "", "");
        opts
    };
    
    static ref PGM_OPTS: getopts::Matches
			  = PGM_OPTIONS.parse( PGM_ARGS.iter() ).
				unwrap_or_else( |err| {
      		  std::env::set_exit_status(1);
      		  panic!(err.to_string());
   			} );
}                              // lazy_static!

// fetch options which have a default

type OptStr = String; // want &'static str !!
// type BufRdr = BufReader<&[u8]>;

fn opt_str(opt_name: &str, dfalt: &str)->OptStr {
  PGM_OPTS.opt_str(opt_name).unwrap_or(dfalt.to_string()) // to_string !!
}
fn opt_u16(opt_name: &str, dfalt: u16)->u16 {
  match PGM_OPTS.opt_str(opt_name) {
    None => dfalt,
    Some(p) => p.parse::<u16>().unwrap_or_else( |err| {
      std::env::set_exit_status(2);
      panic!(err.to_string());
    } )
  }
}
fn http_port()->u16 { opt_u16("http-port", 8080) }
fn db_init_func()->OptStr { opt_str("init-func", "wicci_ready") }
fn db_host()->OptStr { opt_str("db-host", "localhost") }
fn db_port()->u16 { opt_u16("db-port", 5432) }
fn db_name()->OptStr { opt_str("db-name", "wicci1") }

// other command-line management

fn print_usage() {
    let brief = format!("Usage: {} [options]...", *PGM_NAME);
    print!("{}", PGM_OPTIONS.usage(&brief));
}

// define database connection pool structures

// Hyper manages workers!
// tiny-http says it does too!

fn str_reader<'a>(s: &'a str)->BufReader<&'a[u8]> { BufReader::new(s.as_bytes()) }

// fn html<'a>(title: &'a str, contents: &str)->BufReader<&'a[u8]> {
fn html(title: &str, contents: &str)->String {
//  str_reader(
    format!("<html>
  <head>
    <title>{0}</title>
  </head>
  <body>
    <h1>{0}</h1>
    {1}
  </body>
</html>", title, contents
    )
//  )
}

fn html_id(id_str: &str)->String { // stricter than standard!
    let re = regex!(r"^[[:alpha:]]+[[:alnum:]]*$");
    assert_eq!(re.is_match(id_str), true);
    id_str.to_ascii_lowercase()
}
fn html_attr(attr_str: &str)->String { html_id(attr_str) }
fn html_tag(tag_str: &str)->String { html_id(tag_str) }
fn html_val(value_str: &str)->String { // stricter than standard!
    let re = regex!(r"^[[:graph:] ]*$"); // spaces allowed!
    assert_eq!(re.is_match(value_str), true);
    let quote = regex!("\"");
    quote.replace_all(value_str, "&quot;")
}
fn html_text(text: &str)->String {
    // should translate illegal chars!!
    text.to_string()            // to_string() :( !!
}
fn html_format(text: fmt::Arguments)->String {
    // should translate illegal chars!!
    html_text(&format!("{}", text))
}

// fn html_attrs(attrs: &[&str])-> String {
//   attrs.chunks(2).map( |pair| {
//     format!(" {}=\"{}\"", html_attr(pair[0]), html_val(pair[1]))
//   } ).concat()
// }

fn html_attrs(attrs: &[&str])-> String {
  let mut buf = String::new();
  for pair in attrs.chunks(2) {
    write!(&mut buf, " {}=\"{}\"", html_attr(pair[0]), html_val(pair[1]));
  }
  buf
}

fn html_elem(tag: &str, attrs: &[&str], contents: &[&str])-> String {
  format!("<{0}{1}>\n{2}\n</{0}>\n",
          html_tag(tag), html_attrs(attrs),
          contents.concat() )
}

fn html_elm(tag: &str, contents: &[&str])-> String {
  format!("<{0}>\n{1}\n</{0}>\n",
          html_tag(tag), contents.concat() )
}

fn html_el(tag: &str, contents: &str)-> String {
  format!("<{0}>\n{1}\n</{0}>\n",
          html_tag(tag), contents)
}

fn main() {
  if PGM_OPTS.opt_present("help") { print_usage(); }
	let port = http_port();
  let get_ = tiny_http::Method::from_str("GET").unwrap();
  let put_ = tiny_http::Method::from_str("PUT").unwrap();
  // let get_ = tiny_http::Method(AsciiStr::from_str("GET").unwrap());
  // let put_ = tiny_http::Method(AsciiStr::from_str("PUT").unwrap());
	let server = tiny_http::ServerBuilder::new().
		with_port(port).build().unwrap_or_else( | err | {
      std::env::set_exit_status(3);
      panic!(err.to_string());
		});
	for r in server.incoming_requests() {
    if r.get_url() == "/favicon.ico" {
      let headers: Vec<tiny_http::Header> = Vec::new();
//      let data_reader = BufReader::new("".as_bytes());
      r.respond(tiny_http::Response::new(
        tiny_http::StatusCode::from(404), headers, str_reader(""), None, None
//     tiny_http::StatusCode::from(404), headers, data_reader, None, None
      ));
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
//      let html_reader = BufReader::new("".as_bytes());
      let html_str = html("shim response",
        &html_elm(
          "dl", &[
            &html_el("dt", &html_text("method")),
            &html_el("dd", &format!("{}", r.get_method())),
            &html_el("dt", &html_text("url")),
            &html_el("dd", &format!("{}", r.get_url())),
            &html_el("dt", &html_text("http_version")),
            &html_el("dd", &format!("{}", r.get_http_version())),
            &header_data
          ]
        )
      );
      let html_reader = str_reader( &html_str );
      r.respond(tiny_http::Response::new(
        tiny_http::StatusCode::from(200), headers,
        html_reader,
        None, None ));
    }
    println!("");
  }
}

