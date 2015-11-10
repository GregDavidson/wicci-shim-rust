// Wicci Shim Module
// Options & Argument Management

// #![feature(macro_rules)]
// mod macros;

use std::str::FromStr;
use std::process;

const NUM_WORKERS_DEFAULT: usize = 4;
const HTTP_PORT_DEFAULT: u16 = 8080;
const DB_PORT_DEFAULT: u16 = 5432;
const DB_HOST_DEFAULT: &'static str = "localhost";
const DB_NAME_DEFAULT: &'static str = "wicci1";
const DB_INIT_FUNC_DEFAULT: &'static str = "wicci_ready";
const DB_FUNC_DEFAULT: &'static str = "wicci_serve";
const DB_USER_DEFAULT_DEFAULT: &'static str
  = "greg"; // see DB_USER_DEFAULT below
                 
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
    Some(p) => T::from_str(&p).unwrap_or_else( |_| {
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

pub fn wicci_init_sql(fn_name: &str) -> String {
  //  Need to sql_literal(fn_name)!!!
  format!("SELECT {}($1)", fn_name)
}

pub fn wicci_sql(fn_name: &str) -> String {
  //  Need to sql_literal(fn_name)!!!
  format!("SELECT h,v,b FROM {}($1, $2, $3) AS foo(h,v,b)", fn_name)
}

pub fn db_connect_str(
  host: &str, port: u16,
  name: &str, user: &str, pw: &str,
) -> String {
  let have_pw = pw != "";
  let have_port = port != 0;
  format!(
    "postgresql://{}{}{}@{}{}{}/{}",
    user,
    if have_pw { ":" } else { "" },
    if have_pw { pw } else { "" },
    host,
    if have_port { ":" } else { "" },
    if have_port { port } else { 8080 },
    name
  )
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
	  opts.optopt("I", "db-init-func", "", "name of sql init func"); // DB_INIT_FUNC_DEFAULT
	  opts.optopt("F", "db-func", "", "name of sql service function"); // DB_FUNC_DEFAULT
	  // db connection attributes: see DBOption
	  opts.optopt("H", "db-host", "", "db server host"); // DB_HOST_DEFAULT
    // opts.optopt("A", "db-host-addr", "", ""); // allow numeric db-host??
    opts.optopt("P", "db-port", "", "db server port"); // DB_PORT_DEFAULT
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
  
  pub static ref DBUG: bool
    = PGM_OPTS.opt_present("debug");
  static ref DB_USER_DEFAULT: String
    = ::std::env::var("USER")
      .unwrap_or(DB_USER_DEFAULT_DEFAULT.to_string());
  pub static ref NUM_WORKERS: usize =
    opt_default::<usize>("num-workers", NUM_WORKERS_DEFAULT);
  pub static ref HTTP_PORT: u16
    = opt_default::<u16>("http-port", HTTP_PORT_DEFAULT);
  pub static ref DB_INIT_FUNC: OptStr =
    opt_str_default("db-init-func", DB_INIT_FUNC_DEFAULT);
  pub static ref DB_INIT_STMT: String
    = wicci_init_sql(&*DB_INIT_FUNC);
  static ref DB_FUNC: OptStr
    = opt_str_default("db-func", DB_FUNC_DEFAULT);
  pub static ref WICCI_SERVE_SQL: String
    = wicci_sql(&*DB_FUNC);
  pub static ref DB_HOST: OptStr
    = opt_str_default("db-host", DB_HOST_DEFAULT);
  pub static ref DB_PORT: u16
    = opt_default::<u16>("db-port", DB_PORT_DEFAULT);
  pub static ref DB_USER: OptStr
    = opt_str_default("db-user", &*DB_USER_DEFAULT);
  pub static ref DB_PW: OptStr
    = opt_str_default("db-password", "");
  pub static ref DB_NAME: OptStr
    = opt_str_default("db-name", DB_NAME_DEFAULT);
  
  // e.g. "postgresql://greg@localhost/greg";
  pub static ref DB_DSN: String = {
    db_connect_str(
      &*DB_HOST, *DB_PORT,
      &*DB_NAME,
      &*DB_USER, &*DB_PW
    )
  };

}                              // lazy_static!

pub fn print_usage() {
  let brief = format!("Usage: {} [options]...", *PGM_NAME);
  print!("{}", PGM_OPTIONS.usage(&brief));
}

#[cfg(test)]
mod test {
  // use super::*;
  
  #[test]
  fn test1() {
    
  }
}
