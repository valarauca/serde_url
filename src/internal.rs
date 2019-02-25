
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::Path;

use super::errors::UrlFault;

use super::url::Url;
pub use super::url::Host;
use super::url::percent_encoding::{percent_decode};

/// PrivateUrl is a structure which constains the expanded
/// data of a parsed URL
pub struct PrivateUrl {
    url_data: Url,
    string_data: Box<str>,
    username: Option<Box<str>>,
    password: Option<Box<str>>,
    path: Option<Box<str>>,
    full_query: Option<Box<str>>,
    query_key_values: HashMap<String,QueryValues>,
}
impl PrivateUrl {

    /// `new` handles parsing a URL input
    pub fn new(input: &str) -> Result<PrivateUrl,UrlFault> {
        let url_data = Url::parse(input)?;
        let string_data = url_data.to_string().into_boxed_str();
        let username = match boilerplate(url_data.username(), UrlFault::UserNameUtf8) {
            Option::None => None,
            Option::Some(Ok(username)) => Some(username),
            Option::Some(Err(e)) => return Err(e),
        };
        let password = match boilerplate(url_data.password(), UrlFault::PasswordUtf8) {
            Option::None => None,
            Option::Some(Ok(password)) => Some(password),
            Option::Some(Err(e)) => return Err(e),
        };
        let path = match boilerplate(url_data.path(), UrlFault::PathUtf8) {
            Option::None => None,
            Option::Some(Ok(path)) => Some(path),
            Option::Some(Err(e)) => return Err(e),
        };
        let full_query = match boilerplate(url_data.query(), UrlFault::FullQueryUtf8) {
            Option::None => None,
            Option::Some(Ok(path)) => Some(path),
            Option::Some(Err(e)) => return Err(e),
        };
        let query_key_values = url_data.query_pairs()
                .map(|(key,value)| (key.to_string(), value.split("+").collect::<QueryValues>()))
                .collect::<HashMap<String,QueryValues>>();

        Ok(PrivateUrl{ url_data, string_data, username, password, path, full_query, query_key_values } ) 
    }

    /// `get_string` just returns a string
    #[inline(always)]
    pub fn get_string<'a>(&'a self) -> &'a str {
        self.string_data.as_ref()
    }

    /// `get_scheme` returns the URL's scheme
    #[inline(always)]
    pub fn get_scheme<'a>(&'a self) -> &'a str {
        self.url_data.scheme()
    }

    /// `get_username` returns the percentage decoded username
    /// if one is present.
    #[inline(always)]
    pub fn get_username<'a>(&'a self) -> Option<&'a str> {
        self.username.iter().map(|arg| arg.as_ref()).next()
    }

    /// `get_password` returns the percentage decoded password
    /// if one is present.
    #[inline(always)]
    pub fn get_password<'a>(&'a self) -> Option<&'a str> {
        self.password.iter().map(|arg| arg.as_ref()).next()
    }

    /// `get_host` returns host information. This maybe a domain
    /// name, or IP address. You are encouraged to inspect the
    /// value if interested.
    #[inline(always)]
    pub fn get_host<'a>(&'a self) -> Option<Host<&'a str>> {
        self.url_data.host()
    }

    /// `get_port` returns host information about the `port`.
    #[inline(always)]
    pub fn get_port(&self) -> Option<u16> {
        self.url_data.port()
    }

    /// `get_origin` returns an a _non-opaque_ origin. If one
    /// is present. This contains the `host` and `port`, as
    /// well as `scheme` information.
    pub fn get_origin<'a>(&'a self) -> Option<Origin<'a>> {
        self.url_data.host().into_iter()
            .zip(self.url_data.port())
            .map(|(host,port)| Origin {
                scheme: self.url_data.scheme(),
                host: host,
                port: port, 
            })
            .next()
    }

    /// `get_path` returns the `path` component of the URL
    #[inline(always)]
    pub fn get_path<'a>(&'a self) -> Option<&'a Path> {
        self.path.iter().map(|path| Path::new(path.as_ref())).next()
    }

    /// `get_path_str` returns the `path` component of the URL, as a `str` vs `Path`,
    /// which maybe preferable in some scenarios.
    #[inline(always)]
    pub fn get_path_str<'a>(&'a self) -> Option<&'a str> {
        self.path.iter().map(|path| path.as_ref()).next()
    }

    /// `get_query_info` returns information about query parameters
    #[inline(always)]
    pub fn get_query_info<'a>(&'a self) -> Option<QueryData<'a>> {
        match &self.full_query {
            &Option::None => None,
            &Option::Some(ref query) => Some( QueryData {
                full_query: query.as_ref(),
                collection: &self.query_key_values
            })
        }
    }
}

/// QueryData contains information about the URL's query key
/// values. As well as information about the query string
/// itself.
pub struct QueryData<'a> {
    full_query: &'a str,
    collection: &'a HashMap<String,QueryValues>,
}
impl<'a> QueryData<'a> {

    /// `get_full_query` attempts to return the percentage decoded query string
    pub fn get_full_query<'b>(&'b self) -> &'b str {
        self.full_query
    }

    /// `get_exists` checks if a query value exists
    pub fn key_exists<S>(&self, key: &S) -> bool
        where
            S: AsRef<str>
    {
        self.collection.get(key.as_ref()).is_some()
    }

    /// `get_key` returns the value(s) associated with a key.
    /// 
    /// ## Note
    ///
    /// This method may return the strange value `Option::Some(QueryValues::None)`
    /// what this means is that a `key` is present, but it has no values associated
    /// with it, or those values have zero lenght.
    pub fn get_key<'b, S>(&'b self, key: &S) -> Option<&'b QueryValues>
        where
            S: AsRef<str>
    {
        self.collection.get(key.as_ref())
    }
}

/// QueryValues is a way of grouping query values.
///
/// * `None` exists as a query parameter maynot have a value, it may simply be present
/// * `Single` implies no `+` split was useful
/// * `Multiple` implies a `+` split rendered data. Please note: Multiple arguments are not well standardized, so this might not work 100% of the time for you usecase.
pub enum QueryValues {
    None,
    Single(Box<str>),
    Multiple(Box<[Box<str>]>),
}
impl<'a> FromIterator<&'a str> for QueryValues {
    fn from_iter<T>(iter: T) -> Self
        where
            T: IntoIterator<Item = &'a str>
    {
        let data: Vec<Box<str>> = iter.into_iter()
            .filter(|arg| arg.len() > 0)
            .map(|arg| arg.to_string().into_boxed_str())
            .collect();
        if data.len() == 0 {
            QueryValues::None
        } else if data.len() == 1 {
            QueryValues::Single(data[0].clone())
        } else {
            QueryValues::Multiple(data.into_boxed_slice())
        }
    }
}

/// Origin defines a slightly incorrect origin structure
#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct Origin<'a> {
    pub scheme: &'a str,
    pub host: Host<&'a str>,
    pub port: u16,
}

#[inline(always)]
fn boilerplate<'a,T>(input: T, err: UrlFault) 
    -> Option<Result<Box<str>,UrlFault>>
    where
        T: Into<Option<&'a str>>
{
    input.into().into_iter()
        .flat_map(full_details)
        .map(|arg| percent_decode(arg.as_bytes())
            .decode_utf8()
            .map_err(|_| err)
            .map(|decoded| decoded.to_string().into_boxed_str())
        )
        .next()
}

#[inline(always)]
fn full_details<'a>(arg: &'a str) -> Option<&'a str> {
    if arg.len() == 0 { None } else { Some(arg) }
}

