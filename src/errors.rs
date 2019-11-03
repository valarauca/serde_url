
use std::error::Error;
use std::fmt;

/// Returns error related to URL faults
///
/// This trait mostly exists to ensure that we do not recycle
/// errors from the base trait into this crate
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum UrlFault {
    /*
     * types uplifted from `url::ParseError`
     *
     */
    EmptyHost,
    IdnaError,
    InvalidPort,
    InvalidIpv4Address,
    InvalidIpv6Address,
    InvalidDomainCharacter,
    RelativeUrlWithoutBase,
    RelativeUrlWithCannotBeABaseUrlIsABaseUrl,
    SetHostOnCannotBeABaseUrl,
    Overflow,

    /*
     * Internal Errors from internal
     * expansion
     *
     */
    UserNameUtf8,
    PasswordUtf8,
    PathUtf8,
    FullQueryUtf8,
}
impl fmt::Display for UrlFault {
    fn fmt(&self,f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self, self.description())
    }
}
impl Error for UrlFault {
    fn description(&self) -> &'static str {
        match self {
            &UrlFault::EmptyHost => "host does not exist",
            &UrlFault::IdnaError => "domain name or label failed process set, it does not meet validity criteria",
            &UrlFault::InvalidPort => "port value is invalid",
            &UrlFault::InvalidIpv4Address => "ipv4 address is not valid",
            &UrlFault::InvalidIpv6Address => "ipv6 address is not valid",
            &UrlFault::InvalidDomainCharacter => "domain name contains invalid character",
            &UrlFault::RelativeUrlWithoutBase => "not resolve URL relative path",
            &UrlFault::RelativeUrlWithCannotBeABaseUrlIsABaseUrl => "URL states it is not a base URL, but it is a base URL",
            &UrlFault::SetHostOnCannotBeABaseUrl => "URL is a base URL, but cannot be",
            &UrlFault::Overflow => "URL length overflowed while parsing",
            &UrlFault::UserNameUtf8 => "URL contains a username which cannot be represented with UTF8",
            &UrlFault::PasswordUtf8 => "URL contains a password which cannot be represented with UTF8",
            &UrlFault::PathUtf8 => "URL contains a path which cannot be represented with UTF8",
            &UrlFault::FullQueryUtf8 => "URL contains a query string which cannot be represented with UTF8",
        }
    }
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
impl From<url::ParseError> for UrlFault {
    #[inline(always)]
    fn from(err: url::ParseError) -> UrlFault {
        match err {
            url::ParseError::EmptyHost => UrlFault::EmptyHost,
            url::ParseError::IdnaError => UrlFault::IdnaError,
            url::ParseError::InvalidPort => UrlFault::InvalidPort,
            url::ParseError::InvalidIpv4Address => UrlFault::InvalidIpv4Address,
            url::ParseError::InvalidIpv6Address => UrlFault::InvalidIpv6Address,
            url::ParseError::InvalidDomainCharacter => UrlFault::InvalidDomainCharacter,
            url::ParseError::RelativeUrlWithoutBase => UrlFault::RelativeUrlWithoutBase,
            url::ParseError::RelativeUrlWithCannotBeABaseBase => {
                UrlFault::RelativeUrlWithCannotBeABaseUrlIsABaseUrl
            }
            url::ParseError::SetHostOnCannotBeABaseUrl => UrlFault::SetHostOnCannotBeABaseUrl,
            url::ParseError::Overflow => UrlFault::Overflow,
        }
    }
}
