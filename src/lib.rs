
#![allow(dead_code,unused_imports)]
#![allow(clippy::needless_lifetimes,
clippy::option_option,clippy::clone_on_copy,clippy::clone_double_ref)]

use std::str;
use std::fmt;
use std::sync;
use std::hash;
use std::path;
use std::ops;
use std::borrow::{Cow, Borrow, ToOwned};
use std::cmp;

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate url;
extern crate serde;

mod errors;
pub use self::errors::UrlFault;
mod internal;
use self::internal::PrivateUrl;
pub use self::internal::{Origin, Host};

/// Opaque type that can be serialized/deserialized and acts
/// like a string.
///
/// The goal is not URL maniplutation, but rather reading and
/// writing URL's, as well as ensuring a consistent format.
///
/// # Note Usernames/Passwords
///
/// Non-UTF8 usernames and passwords will generate an error.
///
/// # Note Hashing & Equality
///
/// This type implements `std::hash::Hash`, it will use the output
/// of `get_string()` for the purposes of hashing or comparison.
/// Either as a utf8 string, or array of bytes.
#[derive(Clone)]
pub struct Url {
    data: sync::Arc<PrivateUrl>,
}
impl Url {
    /// `new` is a generally entrypoint for constructing a `Url`
    /// also applicable are `from_str` and `serde::Deserialize`
    ///
    /// ```
    /// use serde_url::Url;
    ///
    /// let data = "https://google.com";
    /// let url = match Url::new(&data) {
    ///     Err(e) => panic!("failed to parse url:{} error:{:?}", data, e),
    ///     Ok(url) => url,
    /// };
    /// assert_eq!(url, "https://google.com/");
    /// ```
    pub fn new<S>(input: &S) -> Result<Url, UrlFault>
    where
        S: AsRef<str>,
    {
        let data = sync::Arc::new(PrivateUrl::new(input.as_ref())?);
        Ok(Url { data })
    }

    /// `get_string` returns the normalized URL representation
    ///
    /// ```
    /// use serde_url::Url;
    ///
    /// let data = "https://google.com";
    /// let url = Url::new(&data).unwrap();
    /// assert_eq!(url.get_string(), "https://google.com/");
    /// ```
    ///
    /// This method does not require allocations.
    ///
    /// This representation of the URL will be used for most
    /// common string tasks namely:
    ///
    /// - `AsRef<str>`
    /// - `Deref<str>`
    /// - `Hash`
    /// - `AsRef<[u8]>`
    ///
    /// this allows for a great range of flexiability.
    pub fn get_string<'a>(&'a self) -> &'a str {
        self.data.get_string()
    }

    /// `get_input` returns the input argument
    ///
    /// ```
    /// use serde_url::Url;
    ///
    /// let data = "https://google.com";
    /// let url = Url::new(&data).unwrap();
    /// assert!(data == url.get_input());
    /// assert!(data != url.get_string());
    /// ```
    pub fn get_input<'a>(&'a self) -> &'a str {
        self.data.get_input()
    }

    /// `get_scheme` returns the URL's scheme
    ///
    /// ```
    /// use serde_url::Url;
    ///
    /// let data = "https://google.com";
    /// let url = Url::new(&data).unwrap();
    /// assert!(url.get_scheme() == "https");
    /// ```
    pub fn get_scheme<'a>(&'a self) -> &'a str {
        self.data.get_scheme()
    }

    /// `get_username` returns the percentage decoded username
    /// if one is present.
    ///
    /// ```
    /// use serde_url::Url;
    ///
    /// let data = "https://google.com";
    /// let url = Url::new(&data).unwrap();
    /// assert!(url.get_username() == Option::None);
    /// ```
    ///
    /// If we include a username/password
    ///
    /// ```
    /// use serde_url::Url;
    ///
    /// let data = "https://janedoe:hunter2@google.com";
    /// let url = Url::new(&data).unwrap();
    /// assert!(url.get_username() == Option::Some("janedoe"));
    /// assert!(url.get_password() == Option::Some("hunter2"));
    /// ```
    ///
    /// Escape sequences are automatically handled.
    ///
    /// ```
    /// use serde_url::Url;
    ///
    /// let data = "ftps://john%20doe@google.com";
    /// let url = Url::new(&data).unwrap();
    /// assert!(url.get_username() == Option::Some("john doe"));
    /// assert!(url.get_password() == Option::None);
    /// assert!(url.get_scheme() == "ftps");
    /// ```
    pub fn get_username<'a>(&'a self) -> Option<&'a str> {
        self.data.get_username()
    }

    /// `get_password` returns the password from the username field.
    /// See `get_username` for more information.
    pub fn get_password<'a>(&'a self) -> Option<&'a str> {
        self.data.get_password()
    }

    /// `get_host` returns host information. This maybe a domain
    /// name, or IP address. You are encouraged to inspect the
    /// value if interested.
    ///
    /// ```
    /// use serde_url::{Url,Host};
    /// use std::net::Ipv4Addr;
    ///
    /// let data = "https://192.168.0.1:8080/";
    /// let url = Url::new(&data).unwrap();
    /// match url.get_host().unwrap() {
    ///     Host::Ipv4(ipaddr) => assert!(ipaddr == Ipv4Addr::new(192,168,0,1)),
    ///     _ => panic!("should not occur"),
    /// };
    /// ```
    ///
    /// Also works with IPV6
    ///
    /// ```
    /// use serde_url::{Url,Host};
    /// use std::net::Ipv6Addr;
    ///
    /// let data = "https://[fe80::1]:8080/";
    /// let url = Url::new(&data).unwrap();
    /// match url.get_host().unwrap() {
    ///     Host::Ipv6(ipaddr) => assert!(ipaddr == Ipv6Addr::new(0xfe80,0,0,0,0,0,0,0x01)),
    ///     _ => panic!("should not occur"),
    /// };
    /// ```
    ///
    /// And with domains
    ///
    /// ```
    /// use serde_url::{Url,Host};
    ///
    /// let data = "https://github.com:8080/";
    /// let url = Url::new(&data).unwrap();
    /// match url.get_host().unwrap() {
    ///     Host::Domain(domain) => assert!(domain == "github.com"),
    ///     _ => panic!("should not occur"),
    /// };
    /// ```
    pub fn get_host<'a>(&'a self) -> Option<Host<&'a str>> {
        self.data.get_host()
    }

    /// `get_port` returns host information about the `port`.
    ///
    /// ```
    /// use serde_url::{Url,Host};
    ///
    /// let data = "https://github.com:8080/";
    /// let url = Url::new(&data).unwrap();
    /// assert!(url.get_port().unwrap() == 8080u16);
    /// ```
    pub fn get_port(&self) -> Option<u16> {
        self.data.get_port()
    }

    /// `get_origin` returns an a _non-opaque_ origin. If one
    /// is present. This contains the `host` and `port`, as
    /// well as `scheme` information.
    pub fn get_origin<'a>(&'a self) -> Option<Origin<'a>> {
        self.data.get_origin()
    }

    /// `get_path` returns the `path` component of the URL
    ///
    /// # Note
    ///
    /// attempts to decode the percentage encoding if any
    /// is present.
    pub fn get_path<'a>(&'a self) -> Option<&'a path::Path> {
        self.data.get_path()
    }

    /// `get_path_str` returns the `path` component of the URL, as a `str` vs `Path`,
    /// which maybe preferable in some scenarios.
    ///
    /// # Note
    ///
    /// attempts to decode the percentage encoding if any
    /// is present.
    pub fn get_path_str<'a>(&'a self) -> Option<&'a str> {
        self.data.get_path_str()
    }

    /*
    /// `get_query_info` returns information about query parameters
    pub fn get_query_info<'a>(&'a self) -> Option<QueryData<'a>> {
        self.data.get_query_info()
    }
*/
}

/*
 * One time only standard library stuff
 *
 */
impl hash::Hash for Url {
    #[inline(always)]
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.get_string().hash(state)
    }
}
impl fmt::Debug for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_string())
    }
}
impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_string())
    }
}
impl str::FromStr for Url {
    type Err = UrlFault;
    #[inline(always)]
    fn from_str(s: &str) -> Result<Url, Self::Err> {
        let data = sync::Arc::new(PrivateUrl::new(s)?);
        Ok(Url { data })
    }
}
impl AsRef<Url> for Url {
    #[inline(always)]
    fn as_ref<'a>(&'a self) -> &'a Url {
        self
    }
}
impl ops::Deref for Url {
    type Target = str;
    fn deref<'a>(&'a self) -> &'a str {
        self.get_string()
    }
}
impl PartialEq for Url {
    fn eq(&self, other: &Url) -> bool {
        sync::Arc::ptr_eq(&self.data, &other.data) || self.get_string().eq(other.get_string())
    }
}
impl<'a> PartialEq<&'a Url> for Url {
    fn eq(&self, other: &&Url) -> bool {
        let other: &Url = *other;
        other.eq(self)
    }
}
impl Eq for Url {}
unsafe impl Sync for Url {}
unsafe impl Send for Url {}
impl AsRef<[u8]> for Url {
    #[inline(always)]
    fn as_ref<'a>(&'a self) -> &'a [u8] {
        self.get_string().as_bytes()
    }
}
impl AsRef<str> for Url {
    #[inline(always)]
    fn as_ref<'a>(&'a self) -> &'a str {
        self.get_string()
    }
}
impl PartialEq<[u8]> for Url {
    fn eq(&self, other: &[u8]) -> bool {
        other.eq(self.data.get_string().as_bytes())
    }
}
impl PartialEq<Box<[u8]>> for Url {
    fn eq(&self, other: &Box<[u8]>) -> bool {
        other.as_ref().eq(self.get_string().as_bytes())
    }
}
impl PartialEq<str> for Url {
    fn eq(&self, other: &str) -> bool {
        other.eq(self.get_string())
    }
}
impl<'a> PartialEq<&'a [u8]> for Url {
    fn eq(&self, other: &&[u8]) -> bool {
        let other: &[u8] = *other;
        other.eq(self.get_string().as_bytes())
    }
}
impl<'a> PartialEq<&'a Box<[u8]>> for Url {
    fn eq(&self, other: &&Box<[u8]>) -> bool {
        other.as_ref().eq(self.get_string().as_bytes())
    }
}
impl<'a> PartialEq<&'a str> for Url {
    fn eq(&self, other: &&str) -> bool {
        let other: &str = *other;
        other.eq(self.get_string())
    }
}
impl<'a> PartialEq<&'a Vec<u8>> for Url {
    fn eq(&self, other: &&Vec<u8>) -> bool {
        let other: &Vec<u8> = *other;
        other.as_slice().eq(self.get_string().as_bytes())
    }
}
impl<'a> PartialEq<&'a String> for Url {
    fn eq(&self, other: &&String) -> bool {
        let other: &String = *other;
        other.as_str().eq(self.get_string())
    }
}
impl PartialEq<Vec<u8>> for Url {
    fn eq(&self, other: &Vec<u8>) -> bool {
        other.as_slice().eq(self.data.get_string().as_bytes())
    }
}
impl PartialEq<String> for Url {
    fn eq(&self, other: &String) -> bool {
        other.eq(self.get_string())
    }
}
impl<'a> PartialEq<Cow<'a, [u8]>> for Url {
    fn eq(&self, other: &Cow<'a, [u8]>) -> bool {
        other.as_ref().eq(self.get_string().as_bytes())
    }
}
impl<'a> PartialEq<Cow<'a, str>> for Url {
    fn eq(&self, other: &Cow<'a, str>) -> bool {
        other.as_ref().eq(self.get_string())
    }
}
impl<'a> PartialEq<&'a Cow<'a, [u8]>> for Url {
    fn eq(&self, other: &&Cow<'a, [u8]>) -> bool {
        other.as_ref().eq(self.get_string().as_bytes())
    }
}
impl<'a> PartialEq<&'a Cow<'a, str>> for Url {
    fn eq(&self, other: &&Cow<'a, str>) -> bool {
        other.as_ref().eq(self.get_string())
    }
}

impl PartialOrd<[u8]> for Url {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        other.partial_cmp(self.get_string().as_bytes())
    }
}
impl PartialOrd<str> for Url {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        other.partial_cmp(self.get_string())
    }
}
impl PartialOrd<Box<[u8]>> for Url {
    fn partial_cmp(&self, other: &Box<[u8]>) -> Option<cmp::Ordering> {
        other.as_ref().partial_cmp(self.get_string().as_bytes())
    }
}
impl<'a> PartialOrd<&'a [u8]> for Url {
    fn partial_cmp(&self, other: &&[u8]) -> Option<cmp::Ordering> {
        let other: &[u8] = *other;
        other.partial_cmp(self.get_string().as_bytes())
    }
}
impl<'a> PartialOrd<&'a str> for Url {
    fn partial_cmp(&self, other: &&str) -> Option<cmp::Ordering> {
        let other: &str = *other;
        other.partial_cmp(self.get_string())
    }
}
impl<'a> PartialOrd<&'a Box<[u8]>> for Url {
    fn partial_cmp(&self, other: &&Box<[u8]>) -> Option<cmp::Ordering> {
        other.as_ref().partial_cmp(self.get_string().as_bytes())
    }
}
impl<'a> PartialOrd<&'a Vec<u8>> for Url {
    fn partial_cmp(&self, other: &&Vec<u8>) -> Option<cmp::Ordering> {
        let other: &Vec<u8> = *other;
        other.as_slice().partial_cmp(self.get_string().as_bytes())
    }
}
impl<'a> PartialOrd<&'a String> for Url {
    fn partial_cmp(&self, other: &&String) -> Option<cmp::Ordering> {
        let other: &String = *other;
        other.as_str().partial_cmp(self.get_string())
    }
}
impl PartialOrd<Vec<u8>> for Url {
    fn partial_cmp(&self, other: &Vec<u8>) -> Option<cmp::Ordering> {
        other.as_slice().partial_cmp(self.get_string().as_bytes())
    }
}
impl PartialOrd<String> for Url {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        other.as_str().partial_cmp(self.get_string())
    }
}
impl<'a> PartialOrd<Cow<'a, [u8]>> for Url {
    fn partial_cmp(&self, other: &Cow<'a, [u8]>) -> Option<cmp::Ordering> {
        other.as_ref().partial_cmp(self.get_string().as_bytes())
    }
}
impl<'a> PartialOrd<Cow<'a, str>> for Url {
    fn partial_cmp(&self, other: &Cow<'a, str>) -> Option<cmp::Ordering> {
        other.as_ref().partial_cmp(self.get_string())
    }
}
impl<'a> PartialOrd<&'a Cow<'a, [u8]>> for Url {
    fn partial_cmp(&self, other: &&Cow<'a, [u8]>) -> Option<cmp::Ordering> {
        other.as_ref().partial_cmp(self.get_string().as_bytes())
    }
}
impl<'a> PartialOrd<&'a Cow<'a, str>> for Url {
    fn partial_cmp(&self, other: &&Cow<'a, str>) -> Option<cmp::Ordering> {
        other.as_ref().partial_cmp(self.get_string())
    }
}

/*
 * Serde Serialize
 *
 * Here we describe how a URL is serialized. Spoiler,
 * it is a string.
 */
impl serde::Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.data.as_ref().get_string())
    }
}

/*
 *
 * Serde DeSerialize
 *
 * Here we define a `Visitor` which lets tell serde
 * what the type is, and how to deserialize that
 * type.
 *
 */
struct UrlVisitor;
impl<'de> serde::de::Visitor<'de> for UrlVisitor {
    type Value = Url;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Url")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        use std::str::FromStr;

        Url::from_str(value)
            .map_err(|e| format!("{:?}", e))
            .map_err(serde::de::Error::custom)
    }
}

/*
 * Serde DeSerialize
 *
 * Here we actually define `Deserialize` the trait
 * we just give serde the vistor.
 *
 * In reality the visitor has no size, so it can't
 * we passed to a function, and magic happens at
 * compile time, kind of.
 */
impl<'de> serde::Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UrlVisitor)
    }
}
