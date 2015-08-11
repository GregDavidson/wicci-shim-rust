// Wicci Shim Module
// Options & Argument Management

// copy of def in main.rs -- see that one!!
// macro_rules! errorln(
//     ($($arg:tt)*) => (
//       use std::io::Write;
//       match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
//         Ok(_) => {},
//         Err(x) => panic!("Unable to write to stderr: {}", x),
//       }
//     )
// );
use super::errorln;

use std::str::FromStr;
use std::process;

const NUM_WORKERS_DEFAULT: usize = 4;
const HTTP_PORT_DEFAULT: u16 = 8080;
const DB_PORT_DEFAULT: u16 = 5432;
const DB_HOST_DEFAULT: &'static str = "localhost";
const DB_NAME_DEFAULT: &'static str = "wicci1";
const DB_INIT_FUNC_DEFAULT: &'static str = "wicci_ready";
const DB_FUNC_DEFAULT: &'static str = "wicci_serve";
const DB_USER_DEFAULT_DEFAULT: &'static str = "greg"; // see DB_USER_DEFAULT below

// fetch options which have a default

/* I'd rather not use String or any other heap-allocated
 * types.  Failing that, I'll try to make such objects less
 * ephemeral, e.g. lazy static.  Failing that, I'll try to
 * make them very ephemeral so that it might be possible to
 * stack-allocate them in the future!
 */

pub type OptStr = String;

pub fn opt_str(opt_name: &str) -> Option<OptStr> {
  PGM_OPTS.opt_str(opt_name)
}

pub fn opt_default<T: FromStr>(opt_name: &str, dfalt: T)->T {
  match PGM_OPTS.opt_str(opt_name) {
    None => dfalt,
    Some(p) => T::from_str(&p).unwrap_or_else( |err| {
      // log failure!!
      // shutdown server gracefully!!
      // why does this not compile?? :
      // errorln!( "opt_default invalid option name {:?}, value: {:?}, error: {:?}",
      //            opt_name, &p, err );
      errorln!( "opt_default failed on {}", opt_name );
      process::exit(20);
    } )
  }
}

pub fn opt_str_default(opt_name: &str, dfalt: &str)->String {
  opt_default::<String>(opt_name, dfalt.to_string())
}

pub fn opt_present(opt_name: &str) -> bool {
  PGM_OPTS.opt_present(opt_name)
}

lazy_static! {
  static ref PGM_ARGV: Vec<String> = {
    let argv = ::std::env::args().collect();
    argv
  };
  pub static ref PGM_NAME: String = PGM_ARGV[0].clone();
  static ref PGM_ARGS: &'static[String] = &PGM_ARGV[1..];
  
  static ref PGM_OPTIONS: ::getopts::Options = {
    let mut opts = ::getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("d", "debug", "display what's going on");
    opts.optopt("t", "test", "", "test name");
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
  
  static ref PGM_OPTS: ::getopts::Matches
		= PGM_OPTIONS.parse( PGM_ARGS.iter() ).
		unwrap_or_else( |err| {
          // log failure!!
          // shutdown server gracefully!!
          errorln!("program options parse error {:?}", err);
          process::exit(21);
   	} );
  
  pub static ref DBUG: bool = PGM_OPTS.opt_present("debug");
  
  static ref DB_USER_DEFAULT: String
    = ::std::env::var("USER").unwrap_or(DB_USER_DEFAULT_DEFAULT.to_string());
  
  pub static ref NUM_WORKERS: usize =
    opt_default::<usize>("num-workers", NUM_WORKERS_DEFAULT);
  pub static ref HTTP_PORT: u16 = opt_default::<u16>("http-port", HTTP_PORT_DEFAULT);
  pub static ref DB_INIT_FUNC: OptStr =
    opt_str_default("db-init-func", DB_INIT_FUNC_DEFAULT);
  pub static ref DB_INIT_STMT: String =
    format!("SELECT \"{}\"($1)", *DB_INIT_FUNC); /*
  Need to sql_literal(DB_INIT_FUNC) and sql_quote(PGM_NAME)!!!
   */
  static ref DB_FUNC: OptStr = opt_str_default("db-func", DB_FUNC_DEFAULT);
  pub static ref DB_QUERY_STR: String =
    format!("SELECT h,v,b FROM {}($1, '_body_bin') AS foo(h,v,b)",
            *DB_FUNC);   //  Need to sql_literal(DB_FUNC)!!!
  pub static ref DB_HOST: OptStr = opt_str_default("db-host", DB_HOST_DEFAULT);
  pub static ref DB_PORT: u16 = opt_default::<u16>("db-port", DB_PORT_DEFAULT);
  pub static ref DB_USER: OptStr = opt_str_default("db-user", &*DB_USER_DEFAULT);
  pub static ref DB_NAME: OptStr = opt_str_default("db-name", DB_NAME_DEFAULT);
  
  // e.g. "postgresql://greg@localhost/greg";
  pub static ref DB_DSN: String = {
    let pw = PGM_OPTS.opt_present("db-password");
    format!(
      "postgresql://{}{}{}@{}/{}", *DB_USER,
      if pw { ":" } else { "" },
      if pw { PGM_OPTS.opt_str("db-password").unwrap() } else { "".to_string() },
      *DB_HOST, *DB_NAME )
  };
  
}                              // lazy_static!

pub fn print_usage() {
  let brief = format!("Usage: {} [options]...", *PGM_NAME);
  print!("{}", PGM_OPTIONS.usage(&brief));
}

#[cfg(test)]
mod test {
  use super::*;
  
  #[test]
  fn basics_int() {
    
  }
}
