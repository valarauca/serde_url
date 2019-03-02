
use std::str;
use std::fmt;
use std::sync;
use std::hash;
use std::path;
use std::ops;

extern crate url;
extern crate serde;

mod errors;
pub use self::errors::{UrlFault};
mod internal;
use self::internal::PrivateUrl;
pub use self::internal::{Origin,Host,QueryData};

/// Opaque type that can be serialized/deserialized and acts
/// like a string. 
///
/// The goal is not URL maniplutation, but rather reading and
/// writing URL's, as well as ensuring a consistent format.
///
/// # Note Percentage Encoding
///
/// Fields returned from this interface will be percentage encoding.
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
    data: sync::Arc<PrivateUrl>
}
impl Url {

    /// `new` is a generally entrypoint for constructing a `Url`
    /// also applicable are `from_str` and `serde::Deserialize`
    pub fn new<S>(input: &S) -> Result<Url,UrlFault>
        where
            S: AsRef<str>
    {
        Ok( Url {
            data: sync::Arc::new(PrivateUrl::new(input.as_ref())?),
        }) 
    }

    /// `get_string` just returns a string
    pub fn get_string<'a>(&'a self) -> &'a str {
        self.data.get_string()
    }

    /// `get_input` returns the input argument
    pub fn get_input<'a>(&'a self) -> &'a str {
        self.data.get_input()
    }

    /// `get_scheme` returns the URL's scheme
    pub fn get_scheme<'a>(&'a self) -> &'a str {
        self.data.get_scheme()
    } 

    /// `get_username` returns the percentage decoded username
    /// if one is present.
    pub fn get_username<'a>(&'a self) -> Option<&'a str> {
        self.data.get_username()
    }

    /// `get_password` returns the percentage decoded password
    /// if one is present.
    pub fn get_password<'a>(&'a self) -> Option<&'a str> {
        self.data.get_password()
    }

    /// `get_host` returns host information. This maybe a domain
    /// name, or IP address. You are encouraged to inspect the
    /// value if interested.
    pub fn get_host<'a>(&'a self) -> Option<Host<&'a str>> {
        self.data.get_host()
    }

    /// `get_port` returns host information about the `port`.
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

    /// `get_query_info` returns information about query parameters
    pub fn get_query_info<'a>(&'a self) -> Option<QueryData<'a>> {
        self.data.get_query_info()
    }
}
impl AsRef<Url> for Url {
    #[inline(always)]
    fn as_ref<'a>(&'a self) -> &'a Url {
        self
    }
}
impl AsRef<[u8]> for Url {
    #[inline(always)]
    fn as_ref<'a>(&'a self) -> &'a [u8] {
        self.get_string().as_bytes()
    }
}
impl hash::Hash for Url {
    #[inline(always)]
    fn hash<H>(&self, state: &mut H) 
        where
            H: hash::Hasher
    {
        self.data.as_ref().get_string().hash(state)
    }
}
impl fmt::Debug for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data.as_ref().get_string())
    }
}
impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.data.as_ref().get_string())
    }
}
impl str::FromStr for Url {
    type Err = UrlFault;
    #[inline(always)]
    fn from_str(s: &str) -> Result<Url, Self::Err> {
        let data = sync::Arc::new(PrivateUrl::new(s)?);
        Ok( Url{ data } )
    }
}
impl PartialEq for Url {
    fn eq(&self, other: &Url) -> bool {
        sync::Arc::ptr_eq(&self.data,&other.data) ||
        self.data
            .as_ref()
            .get_string()
            .eq(other.data.as_ref().get_string())
    }
}
impl PartialEq<String> for Url {
    fn eq(&self, other: &String) -> bool {
        other.eq(self.data.as_ref().get_string())
    }
}
impl PartialEq<Vec<u8>> for Url {
    fn eq(&self, other: &Vec<u8>) -> bool {
        other.as_slice().eq(self.data.get_string().as_bytes())
    }
}
impl PartialEq<str> for Url {
    fn eq(&self, other: &str) -> bool {
        other.eq(self.data.as_ref().get_string())
    }
}
impl PartialEq<[u8]> for Url {
    fn eq(&self, other: &[u8]) -> bool {
        other.eq(self.data.get_string().as_bytes())
    }
}
impl<'a> PartialEq<::std::borrow::Cow<'a,str>> for Url {
    fn eq(&self, other: &::std::borrow::Cow<'a,str>) -> bool {
        other.as_ref().eq(self.data.get_string())
    }
}
impl<'a> PartialEq<::std::borrow::Cow<'a,[u8]>> for Url {
    fn eq(&self, other: &::std::borrow::Cow<'a,[u8]>) -> bool {
        other.as_ref().eq(self.data.get_string().as_bytes())
    }
}
impl AsRef<str> for Url {
    fn as_ref<'a>(&'a self) -> &'a str {
        self.data.as_ref().get_string()
    }
}
impl ops::Deref for Url {
    type Target = str;
    fn deref<'a>(&'a self) -> &'a str {
        self.data.get_string()
    }
}
impl Eq for Url { }
unsafe impl Sync for Url { }
unsafe impl Send for Url { }


/*
 * Serde Serialize
 *
 * Here we describe how a URL is serialized. Spoiler,
 * it is a string.
 */
impl serde::Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where 
            S: serde::Serializer
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
    fn visit_str<E>(self, value: &str) -> Result<Self::Value,E>
        where
            E: serde::de::Error
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
            D: serde::Deserializer<'de>
    {
        deserializer.deserialize_str(UrlVisitor)
    }
}

