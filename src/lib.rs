
use std::str;
use std::fmt;
use std::sync;
use std::hash;
use std::path;

extern crate url;
extern crate serde;

mod errors;
pub use self::errors::{UrlFault};
mod internal;
use self::internal::PrivateUrl;
pub use self::internal::{Origin,Host,QueryData,QueryValues};

/// `UrlOp` allows for exposing all the functionality of
/// a `Url` type without exposing internal field
/// semantics. 
///
/// This lets types _inheriate_ `Url` functionality
/// for the fee of just storing a `Url` field, and writing
/// the `impl AsRef<Url` boilerplate.
pub trait UrlOp: AsRef<Url> {

    /// `get_string` just returns a string
    fn get_string<'a>(&'a self) -> &'a str {
        self.as_ref().data.get_string()
    }

    /// `get_scheme` returns the URL's scheme
    fn get_scheme<'a>(&'a self) -> &'a str {
        self.as_ref().data.get_scheme()
    } 

    /// `get_username` returns the percentage decoded username
    /// if one is present.
    fn get_username<'a>(&'a self) -> Option<&'a str> {
        self.as_ref().data.get_username()
    }

    /// `get_password` returns the percentage decoded password
    /// if one is present.
    fn get_password<'a>(&'a self) -> Option<&'a str> {
        self.as_ref().data.get_password()
    }

    /// `get_host` returns host information. This maybe a domain
    /// name, or IP address. You are encouraged to inspect the
    /// value if interested.
    fn get_host<'a>(&'a self) -> Option<Host<&'a str>> {
        self.as_ref().data.get_host()
    }

    /// `get_port` returns host information about the `port`.
    fn get_port(&self) -> Option<u16> {
        self.as_ref().data.get_port()
    }

    /// `get_origin` returns an a _non-opaque_ origin. If one
    /// is present. This contains the `host` and `port`, as
    /// well as `scheme` information.
    fn get_origin<'a>(&'a self) -> Option<Origin<'a>> {
        self.as_ref().data.get_origin()
    }

    /// `get_path` returns the `path` component of the URL
    ///
    /// # Note
    ///
    /// attempts to decode the percentage encoding if any
    /// is present.
    fn get_path<'a>(&'a self) -> Option<&'a path::Path> {
        self.as_ref().data.get_path()
    }

    /// `get_path_str` returns the `path` component of the URL, as a `str` vs `Path`,
    /// which maybe preferable in some scenarios.
    ///
    /// # Note
    ///
    /// attempts to decode the percentage encoding if any
    /// is present.
    fn get_path_str<'a>(&'a self) -> Option<&'a str> {
        self.as_ref().data.get_path_str()
    }

    /// `get_query_info` returns information about query parameters
    fn get_query_info<'a>(&'a self) -> Option<QueryData<'a>> {
        self.as_ref().data.get_query_info()
    }
}

/// Url is an opque type for deserializing, and serializing Url's.
/// Internally it uses an `Arc` to avoid additionally memory usage
/// when cloned. Furthermore `fmt::Debug`, `fmt::Display`, and
/// `hash::Hash` don't trigger additional allocations which a
/// naive implementation may incurr accidently.
///
/// This means creating a new `Url` can cause a number of
/// background allocations, but generally provided the value
/// sticks around for a little while this is beneficial.
///
/// # Note Percentage Encoding
///
/// Fields returned from this API will generally already be
/// percentage decoded.
///
/// # Note
///
/// `UrlOp` is a trait which implements most of the common
/// `Url` operations. It is broken out into a trait, so that
/// other types may expose similiar methods to `Url` while
/// not necessarily exposing their encapsulated data layout.
///
/// In this way a library may accept `AsRef<UrlOp>` instead
/// of the opaque `Url` data type. This promotes better
/// borrow handler behavior, and avoids re-parsing and
/// and allocating Url's which are passed as `String`.
///
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
}
impl AsRef<Url> for Url {
    #[inline(always)]
    fn as_ref<'a>(&'a self) -> &'a Url {
        self
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
impl PartialEq<str> for Url {
    fn eq(&self, other: &str) -> bool {
        other.eq(self.data.as_ref().get_string())
    }
}
impl AsRef<str> for Url {
    fn as_ref<'a>(&'a self) -> &'a str {
        self.data.as_ref().get_string()
    }
}
impl Eq for Url { }
unsafe impl Sync for Url { }
unsafe impl Send for Url { }
impl UrlOp for Url { }


/*
 * Public Serde Stuff
 *
 */
impl serde::Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where 
            S: serde::Serializer
    {
        serializer.serialize_str(&self.data.as_ref().get_string())
    }
}

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
impl<'de> serde::Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>
    {
        deserializer.deserialize_str(UrlVisitor)
    }
}

