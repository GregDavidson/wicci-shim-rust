extern crate tiny_http;
use std::io::Cursor;

pub fn cursor_on<D>(data: D)->Cursor<Vec<u8>> where D: Into<Vec<u8>> {
  Cursor::new(data.into())
}

pub fn str_response(
  status_code: i32, headers: Vec<tiny_http::Header>, str_data: String
    ) -> tiny_http::Response<Cursor<Vec<u8>>> {
  let data_len = str_data.len();
  tiny_http::Response::new(
    tiny_http::StatusCode::from(status_code), headers,
    cursor_on(str_data), Some(data_len), None )
}

pub fn hdr_response(
  status_code: i32, headers: Vec<tiny_http::Header>
    )-> tiny_http::Response<Cursor<Vec<u8>>> {
  tiny_http::Response::new(
    tiny_http::StatusCode::from(status_code), headers,
    cursor_on(Vec::with_capacity(0)), Some(0), None )
}
