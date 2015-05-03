#[macro_use]
extern crate lazy_static;

extern crate getopts;
// use getopts::Options;
use std::env;
use std::sync::{Once, ONCE_INIT};

type str_vec = Vec<String>;
// type str_vec = Vec<&'static str>;
fn pgm_args() -> &str_vec {
    static once: Once = ONCE_INIT;
    static mut argv: str_vec;
    once.call_once( | | argv = *env::args().collect() );
    &argv
}
fn pgm_name() -> String { pgm_args()[0] }

fn get_options(argv: &[String])->getopts::Options {
    static once: Once = ONCE_INIT;
    static mut opts;
    once.call_once( || {
        opts = getopts::Options::new();
        opts.optflag("h", "help", "print this help menu");
        opts.optflag("d", "debug", "trace what's going on");
        opts.optflag("D", "show-args", "show program arguments");
	      opts.optflag("B", "debug-save-blobs", "save received blobs to files");
	      opts.optopt("", "http-port", "", "8080");
	      opts.optopt("", "initfunc", "", "wicci_ready");
	      // db connection attributes: see DBOption
	      opts.optopt("", "host", "", "localhost");
        opts.optopt("", "hostaddr", "", "");
        opts.optopt("", "port", "", "8080");
	      opts.optopt("", "dbname", "", "wicci1");
        opts.optopt("", "user", "", "");
	      opts.optopt("", "password", "", "");
	      opts.optopt("", "connect_timeout", "", "");
    } );
    opts
}
fn get_matches(argv: &[String], opts: getopts::Options)
->getopts::Matches {
    static once: Once = ONCE_INIT;
    let matches;
    once.call_once( | | {
        matches = match get_options(argv).parse( argv ) {
            Ok(m) => m,
            Err(f) => {
                env::set_exit_status(1);
                panic!(f.to_string());
            }
        };
    } );
    matches
}

fn print_usage(opts: &getopts::Options) {
    let brief = format!("Usage: {} [options]...", pgm_name());
    print!("{}", opts.usage(&brief));
}

type MatchesBox = Box<getopts::Matches>;
fn pgm_opts() -> getopts::Matches {
    static once: Once = ONCE_INIT;
    let mut opts: MatchesBox;
    once.call_once( | | opts = Box::new( {
        let args = &pgm_args()[1..];
        get_matches(&args, get_options( args ) )
    } ) );
    *opts
}

fn main() {
    if pgm_opts().opt_present("help") {
        print_usage( &get_options(&pgm_args()[1..]) )
    }
}
