// Wicci Shim Module
// Tests

use std::io::Write;
use super::{options,db};

// copy of def in main.rs -- see that one!!
macro_rules! errorln(
    ($($arg:tt)*) => (
      match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
        Ok(_) => {},
        Err(x) => panic!("Unable to write to stderr: {}", x),
      }
    )
);

use std::process;

use postgres::Statement;

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

fn len_all(bytes: &[&Bytes]) ->usize {
  let mut len: usize = 0;
  for b in bytes.iter() {
    len += b.len();
  }
  len
}

fn cat2vec(byte_vecs: &[&Bytes]) -> ByteVec {
  let mut vec = ByteVec::with_capacity( len_all(byte_vecs) );
  for bytes in byte_vecs.iter() {
    vec.extend( bytes.iter() );
  }
  vec
}

fn path_host_user_server(path: &Bytes, host: &Bytes, user: &Bytes, server: &Bytes) -> ByteVec {
  cat2vec( &[
    b"GET /", path,
    b"?host=",   host,
    b"&",   user,
    b" HTTP/1.1\r\nHost: ",   server,
    b"\r\n\r\n"] )
}

fn path_username_host(path: &Bytes, username: &Bytes, host: &Bytes) -> ByteVec {
  path_host_user_server(path, host, &cat2vec( &[username, b"@", host] ) , SERVER)
}

fn path_req(path: &Bytes) -> ByteVec {
  path_username_host(path, &USERNAME, &HOST)
}

// could define path_req_plus which appends HEADERS

// implement fmt::Display for Bytes and ByteVec !!

fn test_with( stmt: &Statement, req_buf: &Bytes ) {
  let rows = stmt.query(&[&req_buf]).unwrap_or_else( | err | {
    // log error!!
    // send client sad Reponse structure!!
    // continue to next requset!!
    let vec = Vec::from(req_buf);
    let buf = String::from_utf8(vec).unwrap_or(String::from("???"));
    errorln!("test_with db query error {:?} on {}", err, &buf);
    process::exit(40);
  });
  for row in rows {
    let mut hdr_bytes: Vec<u8> = row.get(0);
    super::make_lower_vec_u8(&mut hdr_bytes);    
    let text_bytes:Vec<u8> = row.get(1);
    let bin_bytes:Vec<u8> = row.get(2);
    print!("{:?}:", &hdr_bytes);
    match hdr_bytes.as_slice() {
      b"_body_bin" => {
        println!(" len={} md5={}", bin_bytes.len(), "??" );
        if text_bytes.len() != 0 {
          print!("!! ++ text body:");
          println!(" len={} md5={}\n{:?}", bin_bytes.len(), "??", text_bytes );
        }
      },
      b"_body" => {
        println!("");
        if bin_bytes.len() != 0 {
          print!("+!!+ bin body:");
          println!(" len={} md5={}", bin_bytes.len(), "??" );
        }
        print!(" len={} md5={}\n{:?}", text_bytes.len(), "??", text_bytes );
      },
      _ =>  {
        println!(" {:?}", text_bytes);
        if bin_bytes.len() != 0 {
          print!("+!!+ bin body:");
          println!(" len={} md5={}", bin_bytes.len(), "??" );
        }
      }
    }
  }
}

pub fn do_tests() {
  match options::opt_str("test") {
    None => { },
    Some(test_name) => {
      let db = db::connect();
      let stmt = db::prepare(&db);
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
