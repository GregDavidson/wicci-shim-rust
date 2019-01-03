#![macro_use]

// Evolve to log error and exits
//  with specified status code!!
#[macro_export]
macro_rules! errorln(
    ($($arg:tt)*) => ( {
      use std::io::Write;
      match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
        Ok(_) => {},
        Err(x) => panic!("Unable to write to stderr: {}", x),
      } }
    )
);
