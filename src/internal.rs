
use std::collections::HashMap;
use std::path::Path;
use std::net::{Ipv4Addr, Ipv6Addr, IpAddr, SocketAddr};
use std::fmt::{self, Debug, Display};
use std::hash::{Hash, Hasher};

use super::errors::UrlFault;

use super::url;
use super::url::percent_encoding::percent_decode;

/// PrivateUrl is a structure which constains the expanded
/// data of a parsed URL
pub struct PrivateUrl {
    url_data: url::Url,
    string_data: Box<str>,
    input_data: Box<str>,
    username: Option<Box<str>>,
    password: Option<Box<str>>,
    path: Option<Box<str>>,
    full_query: Option<Box<str>>,
    query_key_values: HashMap<Box<str>, Option<Box<str>>>,
}
impl PrivateUrl {
    /// `new` handles parsing a URL input
    pub fn new(input: &str) -> Result<PrivateUrl, UrlFault> {
        let input_data = input.to_string().into_boxed_str();
        let url_data = url::Url::parse(input)?;
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
        let query_key_values = url_data
            .query_pairs()
            .map(|(key, value)| -> (Box<str>, Option<Box<str>>) {
                let value: Option<Box<str>> = if value.len() != 0 {
                    Some(value.to_string().into_boxed_str())
                } else {
                    None
                };
                let key = key.to_string().into_boxed_str();
                (key, value)
            })
            .collect::<HashMap<Box<str>, Option<Box<str>>>>();

        Ok(PrivateUrl {
            url_data,
            input_data,
            string_data,
            username,
            password,
            path,
            full_query,
            query_key_values,
        })
    }

    /// `get_string` just returns a string
    #[inline(always)]
    pub fn get_string<'a>(&'a self) -> &'a str {
        self.string_data.as_ref()
    }

    /// `get_input` just returns the orginal input string
    #[inline(always)]
    pub fn get_input<'a>(&'a self) -> &'a str {
        self.input_data.as_ref()
    }

    /// `get_scheme` returns the URL's scheme
    #[inline(always)]
    pub fn get_scheme<'a>(&'a self) -> &'a str {
        self.url_data.scheme()
    }

    /// `has_authority()` returns if the URL has an authority
    #[inline(always)]
    pub fn has_authority(&self) -> bool {
        self.url_data.has_authority()
    }

    /// `cannot_be_a_base()` returns if the URL cannot be
    /// a base URL
    #[inline(always)]
    pub fn cannot_be_a_base(&self) -> bool {
        self.url_data.cannot_be_a_base()
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
        match self.url_data.host() {
            Option::Some(url::Host::Ipv4(ref arg)) => Some(Host::Ipv4(arg.clone())),
            Option::Some(url::Host::Ipv6(ref arg)) => Some(Host::Ipv6(arg.clone())),
            Option::Some(url::Host::Domain(ref arg)) => Some(Host::Domain(arg.clone())),
            _ => None,
        }
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
        self.get_host()
            .into_iter()
            .zip(self.url_data.port())
            .map(|(host, port)| {
                Origin {
                    scheme: self.url_data.scheme(),
                    host,
                    port,
                }
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
        match self.full_query {
            Option::None => None,
            Option::Some(ref query) => Some(QueryData {
                full_query: query.as_ref(),
                collection: &self.query_key_values,
            }),
        }
    }
}

/// QueryData contains information about the URL's query key
/// values. As well as information about the query string
/// itself.
pub struct QueryData<'a> {
    full_query: &'a str,
    collection: &'a HashMap<Box<str>, Option<Box<str>>>,
}
impl<'a> QueryData<'a> {
    /// `get_full_query` attempts to return the percentage decoded query string
    pub fn get_full_query<'b>(&'b self) -> &'b str {
        self.full_query
    }

    /// `key_exists` checks if a query value exists
    pub fn key_exists<S>(&self, key: &S) -> bool
    where
        S: AsRef<str>,
    {
        self.collection.get(key.as_ref()).is_some()
    }

    /// `get_key` returns the value(s) associated with a key.
    ///
    /// ## Note
    ///
    /// This method may return the strange value `Option::Some(Some::None)`
    /// what this means is that a `key` is present, but it has no values associated
    /// with it, or those values have zero lenght.
    pub fn get_key<'b, S>(&'b self, key: &S) -> Option<Option<&'b str>>
    where
        S: AsRef<str>,
    {
        match self.collection.get(key.as_ref()) {
            Option::None => None,
            Option::Some(&Option::None) => Some(None),
            Option::Some(&Option::Some(ref arg)) => Some(Some(arg)),
        }
    }
}

/// Host encodes information about host file
pub enum Host<T> {
    Domain(T),
    Ipv4(Ipv4Addr),
    Ipv6(Ipv6Addr),
}
impl<T: Debug> Debug for Host<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Host::Domain(ref arg) => write!(f, "Domain({:?})", arg),
            Host::Ipv4(ref arg) => write!(f, "Ipv4({})", arg),
            Host::Ipv6(ref arg) => write!(f, "Ipv6({})", arg),
        }
    }
}
impl<T: Display> Display for Host<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Host::Domain(ref arg) => write!(f, "Domain({})", arg),
            Host::Ipv4(ref arg) => write!(f, "Ipv4({})", arg),
            Host::Ipv6(ref arg) => write!(f, "Ipv6({})", arg),
        }
    }
}
impl<T: Clone> Clone for Host<T> {
    fn clone(&self) -> Host<T> {
        match self {
            Host::Domain(ref arg) => Host::Domain(arg.clone()),
            Host::Ipv4(ref arg) => Host::Ipv4(arg.clone()),
            Host::Ipv6(ref arg) => Host::Ipv6(arg.clone()),
        }
    }
}
impl<T: Hash> Hash for Host<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        match self {
            Host::Domain(ref arg) => arg.hash(hasher),
            Host::Ipv4(ref arg) => arg.hash(hasher),
            Host::Ipv6(ref arg) => arg.hash(hasher),
        }
    }
}
impl<T: PartialEq> PartialEq for Host<T> {
    fn eq(&self, other: &Host<T>) -> bool {
        match (self, other) {
            (&Host::Domain(ref this), &Host::Domain(ref that)) => this.eq(that),
            (&Host::Ipv4(ref this), &Host::Ipv4(ref that)) => this.eq(that),
            (&Host::Ipv6(ref this), &Host::Ipv6(ref that)) => this.eq(that),
            _ => false,
        }
    }
}
impl<T: Eq> Eq for Host<T> {}

/// Origin defines a slightly incorrect origin structure
#[derive(Clone, Debug)]
pub struct Origin<'a> {
    pub scheme: &'a str,
    pub host: Host<&'a str>,
    pub port: u16,
}
impl<'a> Origin<'a> {
    /// `get_scheme` returns the Origin's scheme
    pub fn get_scheme<'b>(&'b self) -> &'b str {
        self.scheme
    }

    /// `get_port` returns the port
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// `get_socket_addr` returns a network address if the host
    /// IS NOT a domain.
    pub fn get_socket_addr(&self) -> Option<SocketAddr> {
        let addr = match self.host {
            Host::Domain(_) => None,
            Host::Ipv4(ref ipv4) => Some(IpAddr::from(ipv4.clone())),
            Host::Ipv6(ref ipv6) => Some(IpAddr::from(ipv6.clone())),
        };
        addr.into_iter()
            .map(|ip| SocketAddr::from((ip, self.get_port())))
            .next()
    }

    /// `is_domain` checks if this is a domain
    pub fn is_domain(&self) -> bool {
        match self.host {
            Host::Domain(_) => true,
            _ => false,
        }
    }

    /// `get_domain()` returns the domain if this is a domain
    pub fn get_domain<'b>(&'b self) -> Option<&'b str> {
        match self.host {
            Host::Domain(ref domain) => Some(domain),
            _ => None,
        }
    }
}

#[inline(always)]
fn boilerplate<'a, T>(input: T, err: UrlFault) -> Option<Result<Box<str>, UrlFault>>
where
    T: Into<Option<&'a str>>,
{
    input
        .into()
        .into_iter()
        .flat_map(full_details)
        .map(|arg| {
            percent_decode(arg.as_bytes())
                .decode_utf8()
                .map_err(|_| err)
                .map(|decoded| decoded.to_string().into_boxed_str())
        })
        .next()
}

#[inline(always)]
fn full_details<'a>(arg: &'a str) -> Option<&'a str> {
    if arg.is_empty() { None } else { Some(arg) }
}


mod test {

    use super::UrlFault;
    use super::PrivateUrl;

    /*
     * Test Suite Declaration
     *
     */

    struct TestData {
        // input URL to test
        base_url: &'static str,

        // `new` should fail with this error
        error_expected: Option<UrlFault>,

        // `get_string()` should return
        get_string: &'static str,

        // `get_scheme()` should return
        get_scheme: &'static str,

        // `get_username()` should return
        get_username: Option<&'static str>,

        // `get_password` should return
        get_password: Option<&'static str>,
    }
    impl TestData {
        fn validate(&self) -> Result<(), String> {

            // parse output
            let output = match PrivateUrl::new(self.base_url) {
                Ok(output) => output,
                Err(ref e) => {
                    return match self.error_expected {
                        Option::None => Err(format!("{:?} failed to parse {}", e, self.base_url)),
                        Option::Some(ref err) => {
                            if err.eq(e) {
                                Ok(())
                            } else {
                                Err(format!(
                                    "found {:?} not {:?} while parsing {}",
                                    e,
                                    err,
                                    self.base_url
                                ))
                            }
                        }
                    }
                }
            };

            // `get_string` check
            if output.get_string() != self.get_string {
                return Err(format!(
                    "called `get_string()` found {} not {}",
                    output.get_string(),
                    self.get_string
                ));
            }

            // `get_scheme` check
            if output.get_scheme() != self.get_scheme {
                return Err(format!(
                    "called `get_scheme` found {} not {}",
                    output.get_scheme(),
                    self.get_scheme
                ));
            }

            // `get_username` check
            if output.get_username() != self.get_username {
                return Err(format!(
                    "called `get_username` found {:?} not {:?}",
                    output.get_username(),
                    self.get_username
                ));
            }

            // `get_password` check
            if output.get_password() != self.get_password {
                return Err(format!(
                    "called `get_password` found {:?} not {:?}",
                    output.get_password(),
                    self.get_password
                ));
            }

            Ok(())
        }
    }


    #[test]
    fn sanity_check0() {

        let test_data = TestData {
            base_url: "http://google.com/",
            error_expected: None,
            get_string: "http://google.com/",
            get_scheme: "http",
            get_username: None,
            get_password: None,
        };

        test_data.validate().unwrap();
    }
}
