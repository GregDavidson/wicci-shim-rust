#![feature(plugin)]
#![plugin(regex_macros)]
#![feature(exit_status)]  // set_exit_status unstable as of 1.1
#[macro_use]
extern crate lazy_static;
extern crate getopts;
// extern crate hyper; // more than we need?
extern crate tiny_http;
extern crate libc;
extern crate regex_macros;
extern crate regex;

extern crate shim;
use shim::*;

// \/ option and argument management \/

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
    opts.optflag("D", "show-args", "show program arguments");
	  opts.optflag("B", "debug-save-blobs", "save received blobs to files");
    opts.optflag("e", "echo", "echo requests readably");
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

// /\ option and argument management /\


fn main() {
  if PGM_OPTS.opt_present("help") { print_usage(); }
	let port = http_port();
	let server = tiny_http::ServerBuilder::new().
		with_port(port).build().unwrap_or_else( | err | {
      std::env::set_exit_status(3);
      panic!(err.to_string());
		});
  if PGM_OPTS.opt_present("echo") {
    echo_requests(server);
    return;
  }
}
