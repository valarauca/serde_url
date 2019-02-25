
use std::str;
use std::fmt;
use std::sync;
use std::hash;

extern crate url;
extern crate serde;

mod errors;
pub use self::errors::{UrlFault};
mod internal;
use self::internal::PrivateUrl;
pub use self::internal::{Host,QueryData,QueryValues};

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

