// Wicci Shim Module
// Utilities for generating HTML code for debugging & error reporting
// --> Regular HTML should come from the database!

// until regex! is fixed in regex_macros we need
use regex::Regex;

use std::ascii::{AsciiExt};

use std::fmt::{self, Write};

pub type StrVec = Vec<String>;

pub fn html_text(text: String)->String {
  text  // should translate illegal chars!!
}
pub fn html_static(text: &'static str)->String {
  html_text(text.to_string())            // to_string() :( !!
}
pub fn html_format(text: fmt::Arguments)->String {
  html_text(format!("{}", text))
}

pub fn html_id(id_str: &str)->String { // stricter than standard!
  let re = Regex::new(r"^[[:alpha:]]+[[:alnum:]]*$").unwrap();
//  let re = regex!(r"^[[:alpha:]]+[[:alnum:]]*$");
  assert_eq!(re.is_match(&id_str), true);
  id_str.to_ascii_lowercase()
}
pub fn html_attr(attr_str: &str)->String { html_id(attr_str) }
pub fn html_tag(tag_str: &'static str)->String {
  html_id(&tag_str)
}
pub fn html_val(value_str: &str)->String { // stricter than standard!
//  let re = regex!(r"^[[:graph:] ]*$"); // spaces allowed!
  let re = Regex::new(r"^[[:graph:] ]*$").unwrap();
  assert_eq!(re.is_match(&value_str), true);
//  let quote = regex!("\"");
  let quote = Regex::new("\"").unwrap();
  quote.replace_all(&value_str, "&quot;")
}

pub fn html_attrs(attrs: StrVec)-> String {
  let mut buf = String::new();
  for pair in attrs.chunks(2) {
    write!(
      &mut buf, " {}=\"{}\"", html_attr(&pair[0]), &html_val(&pair[1])
        ).unwrap();
    // and if unwrap() fails??
  }
  buf
}

pub fn html_tag_attrs_contents(
  tag: &'static str, attrs: StrVec, contents: StrVec
    )-> String {
  format!("<{0}{1}>\n{2}\n</{0}>\n",
          html_tag(tag), html_attrs(attrs),
          contents.concat() )
}

pub fn html_tag_contents(tag: &'static str, contents: StrVec)-> String {
  html_tag_attrs_contents(tag, Vec::with_capacity(0), contents)
}

pub fn html_tag_content(tag: &'static str, contents: String)-> String {
  html_tag_contents(tag, vec!(contents))
}

pub fn html_title_h1_contents(
  title: Option<&str>, h1: Option<&str>, contents: StrVec
    )->String {
  html_tag_contents( "html", vec!(
    html_tag_content( "head", match title {
      None => "".to_string(),
      Some(s) => html_tag_content("title", s.to_string())
    }),
    html_tag_content( "body", {
      let mut v = vec![ match h1 {
        None => "".to_string(),
        Some(s) => html_tag_content("h1", s.to_string())
      } ];
      v.extend(contents.into_iter());
      v.concat()
    })
      ))
}

pub fn html_title_contents(title_h1: &'static str, contents: String)->String {
  let title_h1_str = html_static(title_h1);
  html_title_h1_contents(
    Some(&title_h1_str), Some(&title_h1_str), vec![contents]
      )
}

#[cfg(test)]
mod test {
//  use super::*;
  
  #[test]
  fn test1() {
    
  }
}
