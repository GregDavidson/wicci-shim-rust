#[macro_use]
extern crate lazy_static;
extern crate getopts;
// extern crate hyper; // more than we need?
extern crate tiny_http;

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
        opts
    };
    
    static ref PGM_OPTS: getopts::Matches
        = match PGM_OPTIONS.parse( PGM_ARGS.iter() ) {
            Ok(m) => m,
            Err(f) => {
                // std::env::set_exit_status(1);
                panic!(f.to_string());
            }
    };
}                              // lazy_static!

fn print_usage() {
    let brief = format!("Usage: {} [options]...", *PGM_NAME);
    print!("{}", PGM_OPTIONS.usage(&brief));
}

// define database connection pool structures

// Hyper manages workers!
// tiny-http says it does too!


fn main() {
  if PGM_OPTS.opt_present("help") {
	  print_usage();
		return;
	}
  let port = 
	let server = tiny_http::ServerBuilder::new().
			with_port(port).build().unwrap();
}
