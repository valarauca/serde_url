
#![allow(dead_code)]

use std::default::Default;

extern crate serde_url;
use serde_url::{Url, Host};

#[derive(Default)]
pub struct UrlTestThing {
    pub url: &'static str,
    pub should_err: bool,
    pub get_string: &'static str,
    pub get_scheme: &'static str,
    pub get_username: Option<&'static str>,
    pub get_password: Option<&'static str>,
    pub get_host: Option<Host<&'static str>>,
    pub get_port: Option<u16>,
    pub get_path_str: Option<&'static str>,
}

fn test(u: &UrlTestThing) -> Result<(), String> {

    // parse
    let url = match Url::new(&u.url) {
        Ok(url) => url,
        Err(ref e) => {
            return if u.should_err {
                Ok(())
            } else {
                Err(format!(
                    "input:({:?}) should not fail with error:({:?})",
                    u.url,
                    e
                ))
            }
        }
    };

    // validate
    if u.get_string != url.get_string() {
        return Err(format!(
            "input:({:?}) has get_string:({:?}) expected:({:?})",
            u.url,
            url.get_string(),
            u.get_string
        ));
    }
    if u.get_scheme != url.get_scheme() {
        return Err(format!(
            "input:({:?}) has get_scheme:({:?}) expected:({:?})",
            u.url,
            url.get_scheme(),
            u.get_scheme
        ));
    }
    if u.get_path_str.is_some() && u.get_path_str != url.get_path_str() {
        return Err(format!(
            "input:({:?}) has get_path_str:({:?}) expected:({:?})",
            u.url,
            url.get_path_str(),
            u.get_path_str
        ));
    }
    if u.get_host.is_some() && u.get_host != url.get_host() {
        return Err(format!(
            "input:({:?}) has get_host:({:?}) expected:({:?})",
            u.url,
            url.get_host(),
            u.get_host
        ));
    }
    if u.get_username.is_some() && u.get_username != url.get_username() {
        return Err(format!(
            "input:({:?}) has get_username:({:?}) expected:({:?})",
            u.url,
            url.get_username(),
            u.get_username
        ));
    }
    Ok(())
}

#[test]
fn regression_test() {

    // test data credit "The Go Authors"

    let test_data: Vec<UrlTestThing> = vec![
        UrlTestThing {
            url: "http://www.google.com",
            should_err: false,
            get_string: "http://www.google.com/",
            get_scheme: "http",
            get_username: None,
            get_password: None,
            get_host: Some(Host::Domain("www.google.com")),
            get_port: None,
            get_path_str: Some("/"),
        },
        UrlTestThing {
            url: "http://www.google.com/file%20one%26two",
            should_err: false,
            get_string: "http://www.google.com/file%20one%26two",
            get_scheme: "http",
            get_username: None,
            get_password: None,
            get_host: Some(Host::Domain("www.google.com")),
            get_port: None,
            get_path_str: Some("/file one&two"),
        },
        UrlTestThing {
            url: "ftp://webmaster@www.google.com/",
            should_err: false,
            get_string: "ftp://webmaster@www.google.com/",
            get_scheme: "ftp",
            get_username: Some("webmaster"),
            get_password: None,
            get_host: Some(Host::Domain("www.google.com")),
            get_port: None,
            get_path_str: Some("/"),
        },
    ];


    for test_item in test_data {
        test(&test_item).unwrap()
    }
}
