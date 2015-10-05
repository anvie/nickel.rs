use std::collections::HashMap;
use request::Request;
use plugin::{Plugin, Pluggable};
use typemap::Key;
use urlencoded;

use std::io::Read;

type FormDataStore = HashMap<String, Vec<String>>;

#[derive(Debug, PartialEq, Eq)]
pub struct FormData(FormDataStore);

impl FormData {
    /// Retrieves the first value from the query for `key`, or `None` if not present.
    ///
    /// # Notes
    /// There may be multiple values per key, if all of the values for a given
    /// `key` are required, then use `all`.
    //FIXME: Implement via Indexing whenever IndexGet is supported
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.first().map(|s| &**s))
    }

    /// Retrieve all values from the query for `key`, or `None` if none are present.
    pub fn all(&self, key: &str) -> Option<&[String]> {
        self.0.get(key).map(|v| &**v)
    }
}

// Plugin boilerplate
struct BodyFormDataParser;
impl Key for BodyFormDataParser { type Value = FormData; }

use std::io;

impl<'mw, 'conn, D> Plugin<Request<'mw, 'conn, D>> for BodyFormDataParser {
    type Error = io::Error;

    fn eval(req: &mut Request<D>) -> Result<FormData, io::Error> {
        // Ok(parse(&req.origin.uri))
        let mut s = String::new();
        try!(req.origin.read_to_string(&mut s));
        Ok(parse(&s))
    }
}

pub trait BodyFormDataString {
    /// Retrieve the query from the current `Request`.
    fn form_data(&mut self) -> &FormData;
}

impl<'mw, 'conn, D> BodyFormDataString for Request<'mw, 'conn, D> {
    fn form_data(&mut self) -> &FormData {
        self.get_ref::<BodyFormDataParser>()
            .ok()
            .expect("Bug: BodyFormDataParser returned None")
    }
}

fn parse(body: &String) -> FormData {
    let f = |d: Option<&String>| d.map(|q| urlencoded::parse(&*q));
    let result = f(Some(&body));
    FormData(result.unwrap_or_else(|| HashMap::new()))
}


#[test]
fn parse_body_form_data() {

    let body_data = "foo=bar&message=hello&message=world".to_string();

    let store = parse(&body_data);
    assert_eq!(store.get("foo"), Some("bar"));
    assert_eq!(store.get("foo").unwrap_or("other"), "bar");
    assert_eq!(store.get("bar").unwrap_or("other"), "other");
    assert_eq!(store.all("message"),
                    Some(&["hello".to_string(), "world".to_string()][..]));
    assert_eq!(store.all("car"), None);
}
