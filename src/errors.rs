
#[derive(Copy,Clone,PartialEq,Eq,PartialOrd,Ord,Hash,Debug)]
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
            url::ParseError::RelativeUrlWithCannotBeABaseBase => UrlFault::RelativeUrlWithCannotBeABaseUrlIsABaseUrl,
            url::ParseError::SetHostOnCannotBeABaseUrl => UrlFault::SetHostOnCannotBeABaseUrl,
            url::ParseError::Overflow => UrlFault::Overflow,
        }
    }
}
