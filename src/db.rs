// Wicci Shim Module
// PostgreSQL Interface

// fn foo(db: &Connection) {
//   let f:() = Connection::connect("", &SslMode::None);
//   let g:() = db.prepare("");
//	core::result::Result<postgres::Statement<'_>, postgres::error::Error>
// }

// * need to define database connection pool structures!!

use std::process;

use postgres::stmt::Statement;
use postgres::{Connection, SslMode};
use postgres::error::ConnectError;
use postgres::Result as PG_Result;
use std::result::Result;

use super::options;


fn try_prepare<'a>(db: &'a Connection, sql_str: &str) -> PG_Result<Statement<'a>> {
  let maybe_sql = db.prepare(sql_str);
  match maybe_sql {
    Ok(_) => (),
    Err(ref e) => if *options::DBUG {
      error!("Preparing query {:?} failed with {:?}", sql_str, e)
    } else { () }
  };
  maybe_sql
}

pub fn prepare_query<'a>(db: &'a Connection, sql_str: &str) -> Statement<'a> {
  let stmt = try_prepare(db, sql_str).unwrap_or_else( | err | {
    // notify clients db is unavailable!!
    // log &alert appropriate sysadmin!!
    // try to continue or shutdown gracefully??
    error!("db prepare statement error {:?}", err);
    process::exit(30);
  });
  stmt
}

pub fn prepare(db: & Connection) -> Statement {
  prepare_query(db, &*options::DB_QUERY_STR)
}

fn try_init(conn: &mut Connection) -> PG_Result<()> {
  let init_stmt = prepare_query(conn, &*options::DB_INIT_STMT);
  // let rows =
  init_stmt.query( &[ &*options::PGM_NAME ] ).unwrap_or_else( |err| {
    error!("db init error {:?}", err);
    process::exit(31);
  });
  // how about we show the result if debugging??
  if *options::DBUG { println!("Session initialized"); }
  Ok(())
}

fn try_connect(dsn: &str) -> Result<Connection, ConnectError> {
  let conn = try!( Connection::connect(dsn, SslMode::None) );
  if *options::DBUG { println!("Connected to PostgreSQL: {}", dsn); }
  Ok(conn)
}

pub fn connect() -> Connection {
  let mut conn = try_connect(&*options::DB_DSN).unwrap_or_else( | err | {
    // notify client db is unavailable??
    // log &alert appropriate sysadmin!!
    // continue to try further requests or shutdown??
    error!("db connect error {:?}", err);
    process::exit(32);
	});
  try_init(&mut conn).unwrap_or_else( | err | {
    // notify client db is unavailable??
    // log &alert appropriate sysadmin!!
    // continue to try further requests or shutdown??
    error!("db init error {:?}", err);
    process::exit(33);
	});
  conn
}
