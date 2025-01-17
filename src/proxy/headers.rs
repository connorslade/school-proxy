use std::borrow::Cow;

use afire::{internal::encoding, Header, HeaderName};
use url::Url;

pub const PROXY_MESSAGE: &str =
    "(Proxied by SchoolProxy [https://github.com/Basicprogrammer10/school-proxy])";
const BLOCKED_HEADERS: &[&str] = &[
    "transfer-encoding",
    "connection",
    "content-security-policy",
    "referrer-policy",
    "content-encoding",
    "accept-encoding",
];

/// To the external server
pub fn transform_header_c2s(header: &Header) -> Option<Cow<Header>> {
    if blocked_header(header) {
        return None;
    }

    match header.name {
        // Rewrite Referrer header to be the original URL
        // https://proxy.connorcode.com/~/https%3A%2F%2Fen.wikipedia.org%2Fwiki%2FMain_Page
        // => https://en.wikipedia.org/wiki/Main_Page
        HeaderName::Referer => return rewrite_referer(&header.value).map(Cow::Owned),
        // Add proxy info to user agent
        HeaderName::UserAgent => return rewrite_user_agent(&header.value).map(Cow::Owned),
        // Remove X-Forwarded-For header
        HeaderName::XForwardedFor => return None,
        _ => {}
    }

    Some(Cow::Borrowed(header))
}

/// To the client from the external server
pub fn transform_header_s2c(mut header: Header, url: &Url) -> Option<Header> {
    if blocked_header(&header) {
        return None;
    }

    match header.name {
        // Rewrite location header to point to proxy address
        HeaderName::Location => {
            if let Ok(url) = url.join(&header.value) {
                header.value = Cow::Owned(format!("/~/{}", encoding::url::encode(url.as_str())));
            }
        }
        _ => {}
    }

    Some(header)
}

// TODO: Referrer policy stuff
// => https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy#directives
fn rewrite_referer(old: &str) -> Option<Header> {
    let url = Url::parse(old).ok()?;
    let path = url.path();
    let (_, path) = path.split_once("/~/")?;
    let path = encoding::url::decode(path);

    Some(Header {
        name: HeaderName::Referer,
        value: Cow::Owned(path),
    })
}

fn rewrite_user_agent(old: &str) -> Option<Header> {
    Some(Header {
        name: HeaderName::UserAgent,
        value: Cow::Owned(format!("{old} {PROXY_MESSAGE}")),
    })
}

fn blocked_header(header: &Header) -> bool {
    BLOCKED_HEADERS.contains(&header.name.to_string().to_ascii_lowercase().as_str())
}
