// Wicci Shim Module
// Tests

use std::io::Write;
use super::db;
use super::options::*;
// use text;

use std::process;
use std::str::from_utf8;

use postgres::stmt::Statement;

type Bytes = [u8];
type ByteVec = Vec<u8>;

static SERVER: &'static [u8] = b"localhost:8080";
static USERNAME: &'static [u8] = b"greg";
static HOST: &'static [u8] = b"wicci.org";

// Test with more headers??
//
// static HEADERS: &[ &[u8] ] = [
//   &b"User-Agent: Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:28.0) Gecko/20100101 Firefox/28.0",
//   &b"Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
//   &b"Accept-Language: en-US,en;q=0.5",
//   &b"Accept-Encoding: gzip, deflate",
//   &b"Connection: keep-alive"
// ];

// // Need to test put requests!!
//
// fn put_path_host_user_server_body(
//   path: &Bytes, host: &Bytes, user: &Bytes, server: &Bytes, body: &Bytes
// ) -> ByteVec {
// }

// error: use of unstable library feature 'iter_arith':
//   bounds recently changed (see issue #27739)
// fn len_all(bytes: &[&Bytes]) ->usize {
//   bytes.iter().map( | x | x.len() ).sum()
// }

fn len_all(bytes: &[&Bytes]) ->usize {
  let mut len: usize = 0;
  for b in bytes.iter() {len += b.len(); }
  len
}

#[test]
fn test_len_all() {
  assert_eq!( len_all(&[]), 0 );
  assert_eq!( len_all(&[b""]), 0 );
  assert_eq!( len_all(&[b"-"]), 1 );
  assert_eq!( len_all(&[b"--"]), 2 );
  assert_eq!( len_all(&[b"", b""]), 0 );
  assert_eq!( len_all(&[b"-", b"-"]), 2 );
}

fn cat2vec(byte_vecs: &[&Bytes]) -> ByteVec {
  let mut vec = ByteVec::with_capacity( len_all(byte_vecs) );
  for bytes in byte_vecs.iter() {
    vec.extend( bytes.iter() );
  }
  vec
}

#[test]
fn test_cat2vec() {
  assert_eq!( cat2vec(&[]), b"" );
  assert_eq!( cat2vec(&[b""]), b"" );
  assert_eq!( cat2vec(&[b"-"]), b"-" );
  assert_eq!( cat2vec(&[b"--"]), b"--" );
  assert_eq!( cat2vec(&[b"", b""]), b"" );
  assert_eq!( cat2vec(&[b"1", b"2"]), b"12" );
}

fn path_host_user_server(path: &Bytes, host: &Bytes, user: &Bytes, server: &Bytes) -> ByteVec {
  cat2vec( &[
    b"GET /", path,
    b"?host=",   host,
    b"&user=",   user,
    b" HTTP/1.1\r\nHost: ",   server,
    b"\r\n"] )
}

fn path_username_host(path: &Bytes, username: &Bytes, host: &Bytes) -> ByteVec {
  path_host_user_server(path, host, &cat2vec( &[username, b"@", host] ) , SERVER)
}

#[test]
fn test_path_username_host() {
  let bv: &[u8]
    = b"GET /P?host=H&user=U@H HTTP/1.1\r\nHost: localhost:8080\r\n";
  assert_eq!( path_username_host(b"P", b"U", b"H"), bv );
}

fn path_req(path: &Bytes) -> ByteVec {
  path_username_host(path, &USERNAME, &HOST)
}

// could define path_req_plus which appends HEADERS

// implement fmt::Display for Bytes and ByteVec !!

fn test_with( stmt: &Statement, req_buf: &Bytes ) {
  if *DBUG { println!("{}", from_utf8(req_buf).unwrap()); }
  let no_body: &Bytes = b"";
  let _body: &str = "_body";
  let rows = stmt.query(&[&req_buf, &no_body, &_body]).unwrap_or_else( | err | {
    // log error!!
    // send client sad Reponse structure!!
    // continue to next request!!
    let buf = from_utf8(req_buf).unwrap();
    errorln!("test_with db query error {:?} on {}", err, buf);
    process::exit(40);
  });
  if *DBUG { println!("Rows returned = {}", rows.len()); }
  for row in rows {
    let hdr: String = row.get(0);
    let text_val: String = row.get_opt(1).unwrap_or("".to_string());
    let bin_val:Vec<u8> = row.get_opt(2).unwrap_or(vec![]);
    print!("{}", hdr);
    if bin_val.len() != 0 {
      print!(" [{}]", bin_val.len());
    }
    match hdr.as_str() {
      "_body" => { println!( ":\n{}", text_val ); },
      _ =>  {
        if text_val != "" {
          println!( ": {}", text_val);
        } else {
            println!( "" );
        }
      }
    }
  }
}

pub fn do_tests() {
  match opt_str("test") {
    None => { },
    Some(test_name) => {
      if *DBUG { println!("Preparing test {}", test_name); }
      let db = db::connect();
      let stmt = db::prepare_query(&db, &*WICCI_SERVE_SQL);
      match &test_name[..] {
        "simple" => test_with( &stmt, &path_req(b"simple") ),
        "fancy" => test_with( &stmt, &path_req(b"fancy") ),
        "jpg" => test_with( &stmt, &path_req(b"Entity-Icon/deadbeef.jpg") ),
        _ => println!("do_tests error: No test named {}", test_name)
      }
    }
  }
}

// To Do:
// * Add representations of expected rows returned
// * Compare return from PostgreSQL with expected rows returned
// * Make --test be a flag which takes args for a battery of concurrent tests
