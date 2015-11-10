// Wicci Shim Module
// PostgreSQL Interface

// fn foo(db: &Connection) {
//   let f:() = Connection::connect("", &SslMode::None);
//   let g:() = db.prepare("");
//	core::result::Result<postgres::Statement<'_>, postgres::error::Error>
// }

use std::io::Write;

// * need to define database connection pool structures!!

use std::process;

use postgres::{Connection, SslMode};
use postgres::stmt::Statement;
use postgres::error::ConnectError;
use postgres::Result as PG_Result;
use std::result::Result;

use super::options;

fn try_prepare<'a>(db: &'a Connection, sql_str: &str) -> PG_Result<Statement<'a>> {
  let maybe_sql = db.prepare(sql_str);
  match maybe_sql {
    Ok(_) => (),
    Err(ref e) => if *options::DBUG {
      errorln!("Preparing query {:?} failed with {:?}", sql_str, e)
    } else { () }
  };
  maybe_sql
}

pub fn prepare_query<'a>(db: &'a Connection, sql_str: &str) -> Statement<'a> {
  let stmt = try_prepare(db, sql_str).unwrap_or_else( | err | {
    // notify clients db is unavailable!!
    // log &alert appropriate sysadmin!!
    // try to continue or shutdown gracefully??
    errorln!("db prepare statement error {:?}", err);
    process::exit(30);
  });
  stmt
}

pub fn prepare_wicci_serve(db: & Connection) -> Statement {
  prepare_query(db, &*options::WICCI_SERVE_SQL)
}

fn try_init(conn: &mut Connection) -> PG_Result<()> {
  let init_stmt = prepare_query(conn, &*options::DB_INIT_STMT);
  let rows =  init_stmt.query( &[ &*options::PGM_NAME ] ).unwrap_or_else( |err| {
    errorln!("db init error {:?}", err);
    process::exit(31);
  });
  assert_eq!(rows.len(), 1);
  // how about we show the result if debugging??
  if *options::DBUG { println!("Session initialized"); }
  Ok(())
}

fn try_connect(dsn: &str) -> Result<Connection, ConnectError> {
  let conn = try!( Connection::connect(dsn, &SslMode::None) );
  if *options::DBUG { println!("Connected to PostgreSQL: {}", dsn); }
  Ok(conn)
}

pub fn connect() -> Connection {
  let mut conn = try_connect(&*options::DB_DSN).unwrap_or_else( | err | {
    // notify client db is unavailable??
    // log &alert appropriate sysadmin!!
    // continue to try further requests or shutdown??
    errorln!("db connect error {:?}", err);
    process::exit(32);
	});
  try_init(&mut conn).unwrap_or_else( | err | {
    // notify client db is unavailable??
    // log &alert appropriate sysadmin!!
    // continue to try further requests or shutdown??
    errorln!("db init error {:?}", err);
    process::exit(33);
	});
  conn
}

#[cfg(test)]
mod tests {
  use super::*;
  use options::*;
  use std::io::Write;
  // use postgres;
  // use postgres::Connection;
  // use postgres::stmt::Statement;
  // use postgres::error::ConnectError;
  // use postgres::Result as PG_Result;

  // use encoding::{Encoding, EncoderTrap};
  // use encoding::all::ISO_8859_1;

  const NO_BYTES: &'static [u8] = b"";
  const HUBBA_BYTES: &'static [u8] = b"Hubba\r\n Hubba\r\n";

  const _BODY: &'static str = "_body";
  const _BODY_BIN: &'static str = "_body_bin";
  const _BODY_LO: &'static str = "_body_lo";

//  lazy_static! {
    // static ref CONNECTION: Connection = connect();
    // pub static ref ECHO_HDRS: Statement<'static>
    //   = prepare_query(*CONNECTION, wicci_sql("wicci_echo_headers"));
    // pub static ref ECHO_BODY: Statement<'static>
    //   = prepare_query(*CONNECTION, wicci_sql("wicci_echo_body"));
    // pub static ref ECHO_REQ: Statement<'static>
    //   = prepare_query(*CONNECTION, wicci_sql("wicci_echo_request"));
//  }
  
  #[test]
  fn echo_prepare() {
    let db = connect();
    let echo_body
       = prepare_query(&db, &wicci_sql("wicci_echo_body"));
    let meta_cols = echo_body.columns();
    let _ = writeln!(
      &mut ::std::io::stderr(), "wicci query column info: {:?}", meta_cols
    );
  }
  #[test]
  fn echo_bin_body() {
    let db = connect();
    let echo_body
       = prepare_query(&db, &wicci_sql("wicci_echo_body"));
    let maybe_rows = echo_body.query(
      &[&NO_BYTES, &HUBBA_BYTES, &_BODY_BIN]
    );
    match maybe_rows {
      Err(msg) => {
        errorln!("Query {} failed with {}", wicci_sql("wicci_echo_body"), msg );
        assert!(false);
      }
      Ok(rows) => {
        assert_eq!(rows.len(), 1);
        let row = rows.get(0);
        assert_eq!(row.len(), 3);
        let _ = writeln!(&mut ::std::io::stderr(), "{:?}", row);
        let field0: String = row.get(0);
        assert_eq!(field0, _BODY_BIN);
        // let _ = writeln!(
        //   &mut ::std::io::stderr(), "{} {}: {}", 0, field0.len(), field0
        // );
        let field1: String = row.get(1);
        assert_eq!(field1.len(), 0);
        // let _ = writeln!(
        //   &mut ::std::io::stderr(), "{} {}: {}", 1, field1.len(), field1
        // );
        let field2: Vec<u8> = row.get(2);
        assert_eq!(field2, HUBBA_BYTES);
        // let _ = writeln!(
        //   &mut ::std::io::stderr(), "{} {}: {:?}", 2, field2.len(), field2
        // );
      }
    }
  }

  #[test]
  fn echo_text_body() {
    let db = connect();
    let echo_body
       = prepare_query(&db, &wicci_sql("wicci_echo_body"));
    let maybe_rows = echo_body.query(
      &[&NO_BYTES, &HUBBA_BYTES, &_BODY]
    );
    match maybe_rows {
      Err(msg) => {
        errorln!("Query {} failed with {}", wicci_sql("wicci_echo_body"), msg );
        assert!(false);
      }
      Ok(rows) => {
        assert_eq!(rows.len(), 1);
        let row = rows.get(0);
        assert_eq!(row.len(), 3);
        let _ = writeln!(&mut ::std::io::stderr(), "{:?}", row);
        let field0: String = row.get(0);
        assert_eq!(field0, _BODY);
        // let _ = writeln!(
        //   &mut ::std::io::stderr(), "{} {}: {}", 0, field0.len(), field0
        // );
        let field1: String = row.get(1);
        assert_eq!(field1.as_bytes(), HUBBA_BYTES);
        // let _ = writeln!(
        //   &mut ::std::io::stderr(), "{} {}: {}", 1, field1.len(), field1
        // );
        let field2: Vec<u8> = row.get(2);
        assert_eq!(field2.len(), 0);
        // let _ = writeln!(
        //   &mut ::std::io::stderr(), "{} {}: {:?}", 2, field2.len(), field2
        // );
      }
    }
  }

}
