#![feature(plugin)]
// #![plugin(regex_macros)]
#![feature(exit_status)]  // set_exit_status unstable as of 1.1
#[macro_use]
extern crate lazy_static;
extern crate getopts;
// extern crate hyper; // more than we need?
extern crate tiny_http;
extern crate libc;
// extern crate regex_macros;
// extern crate regex;

extern crate shim;
use shim::*;
use std::sync::Arc;
use std::thread;
use std::str::FromStr;

// \/ option and argument management \/

const NUM_WORKERS_DEFAULT: usize = 4;
const HTTP_PORT_DEFAULT: u16 = 8080;
const DB_PORT_DEFAULT: u16 = 5432;
const DB_HOST_DEFAULT: &'static str = "localhost";
const DB_NAME_DEFAULT: &'static str = "wicci1";
const DB_INIT_FUNC_DEFAULT: &'static str = "wicci_ready";

// fetch options which have a default

type OptStr = String; // want &'static str !!

// fn opt_str_default(opt_name: &str, dfalt: &str)->OptStr {
//   PGM_OPTS.opt_str(opt_name).unwrap_or(dfalt.to_string()) // to_string !!
// }

// fn opt_num_default<T: FromStr>(opt_name: &str, dfalt: T)->T {
//   match PGM_OPTS.opt_str(opt_name) {
//     None => dfalt,
//     Some(p) => p.parse::<T>().unwrap_or_else( |err| {
//       std::env::set_exit_status(2);
//       panic!(err.to_string());
//     } )
//   }
// }

fn opt_default<T: FromStr>(opt_name: &str, dfalt: T)->T {
  match PGM_OPTS.opt_str(opt_name) {
    None => dfalt,
    Some(p) => T::from_str(&p).unwrap_or_else( |err| {
      std::env::set_exit_status(2);
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
    opts.optflag("D", "show-args", "show program arguments");
	  opts.optflag("B", "debug-save-blobs", "save received blobs to files");
    opts.optflag("e", "echo", "echo requests readably");
	  opts.optopt("W", "num-workers", "", ""); // NUM_WORKERS_DEFAULT
	  opts.optopt("P", "http-port", "", ""); // HTTP_PORT_DEFAULT
	  opts.optopt("F", "db-init-func", "", ""); // DB_INIT_FUNC_DEFAULT
	  // db connection attributes: see DBOption
	  opts.optopt("", "db-host", "", "db server port"); // DB_HOST_DEFAULT
    opts.optopt("", "db-host-addr", "", "");
    opts.optopt("", "db-port", "", ""); // DB_PORT_DEFAULT
	  opts.optopt("", "db-name", "", ""); // DB_NAME_DEFAULT
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

  static ref NUM_WORKERS: usize =
    opt_default::<usize>("num-workers", NUM_WORKERS_DEFAULT);
  static ref HTTP_PORT: u16 = opt_default::<u16>("http-port", HTTP_PORT_DEFAULT);
  static ref DB_INIT_FUNC: OptStr = opt_str_default("init-func", DB_INIT_FUNC_DEFAULT);
  static ref DB_HOST: OptStr = opt_str_default("db-host", DB_HOST_DEFAULT);
  static ref DB_PORT: u16 = opt_default::<u16>("db-port", DB_PORT_DEFAULT);
  static ref DB_NAME: OptStr = opt_str_default("db-name", DB_NAME_DEFAULT);

}                              // lazy_static!

// other command-line management

fn print_usage() {
    let brief = format!("Usage: {} [options]...", *PGM_NAME);
    print!("{}", PGM_OPTIONS.usage(&brief));
}

// /\ option and argument management /\

fn handle_requests(server: Arc<tiny_http::Server>) {
  let mut guards = Vec::with_capacity(*NUM_WORKERS);

  for _ in (0 .. *NUM_WORKERS) {
    let server = server.clone();
    
    let guard = thread::spawn(move || {
      loop {
        let rq = server.recv().unwrap();
        let response = tiny_http::Response::from_string("hello world".to_string());
        rq.respond(response);
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
      std::env::set_exit_status(3);
      panic!(err.to_string());
		});
  if PGM_OPTS.opt_present("echo") {
    echo_requests(server);
    return;
  }
  handle_requests(Arc::new(server));
}
