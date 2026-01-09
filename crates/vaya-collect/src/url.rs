//! URL parsing and manipulation

use crate::{CollectError, CollectResult};

/// A parsed URL
#[derive(Debug, Clone)]
pub struct Url {
    /// Scheme (http or https)
    pub scheme: Scheme,
    /// Host
    pub host: String,
    /// Port (default 80 for http, 443 for https)
    pub port: u16,
    /// Path
    pub path: String,
    /// Query string (without ?)
    pub query: Option<String>,
    /// Fragment (without #)
    pub fragment: Option<String>,
}

/// URL scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}

impl Scheme {
    /// Get the default port for this scheme
    pub fn default_port(&self) -> u16 {
        match self {
            Scheme::Http => 80,
            Scheme::Https => 443,
        }
    }

    /// Get scheme as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Scheme::Http => "http",
            Scheme::Https => "https",
        }
    }
}

impl Url {
    /// Parse a URL string
    pub fn parse(url: &str) -> CollectResult<Self> {
        let url = url.trim();

        // Parse scheme
        let (scheme, rest) = if url.starts_with("https://") {
            (Scheme::Https, &url[8..])
        } else if url.starts_with("http://") {
            (Scheme::Http, &url[7..])
        } else {
            return Err(CollectError::InvalidUrl(format!(
                "Missing or invalid scheme: {}",
                url
            )));
        };

        // Split host from path
        let (authority, path_query_fragment) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };

        // Parse host and port
        let (host, port) = Self::parse_authority(authority, scheme.default_port())?;

        // Parse path, query, and fragment
        let (path, query, fragment) = Self::parse_path_query_fragment(path_query_fragment);

        Ok(Url {
            scheme,
            host,
            port,
            path,
            query,
            fragment,
        })
    }

    /// Parse authority (host:port)
    fn parse_authority(authority: &str, default_port: u16) -> CollectResult<(String, u16)> {
        if authority.is_empty() {
            return Err(CollectError::InvalidUrl("Empty host".into()));
        }

        // Handle IPv6 addresses [...]
        if authority.starts_with('[') {
            if let Some(bracket_end) = authority.find(']') {
                let host = authority[1..bracket_end].to_string();
                let port = if authority.len() > bracket_end + 1 {
                    let port_str = &authority[bracket_end + 2..]; // Skip ]:
                    port_str.parse().map_err(|_| {
                        CollectError::InvalidUrl(format!("Invalid port: {}", port_str))
                    })?
                } else {
                    default_port
                };
                return Ok((host, port));
            }
        }

        // Regular host:port
        match authority.rfind(':') {
            Some(idx) => {
                let host = authority[..idx].to_string();
                let port_str = &authority[idx + 1..];
                let port = port_str
                    .parse()
                    .map_err(|_| CollectError::InvalidUrl(format!("Invalid port: {}", port_str)))?;
                Ok((host, port))
            }
            None => Ok((authority.to_string(), default_port)),
        }
    }

    /// Parse path, query, and fragment
    fn parse_path_query_fragment(s: &str) -> (String, Option<String>, Option<String>) {
        let (path_query, fragment) = match s.find('#') {
            Some(idx) => (&s[..idx], Some(s[idx + 1..].to_string())),
            None => (s, None),
        };

        let (path, query) = match path_query.find('?') {
            Some(idx) => (
                path_query[..idx].to_string(),
                Some(path_query[idx + 1..].to_string()),
            ),
            None => (path_query.to_string(), None),
        };

        (path, query, fragment)
    }

    /// Get the full path with query string for HTTP requests
    pub fn request_path(&self) -> String {
        match &self.query {
            Some(q) => format!("{}?{}", self.path, q),
            None => self.path.clone(),
        }
    }

    /// Get host:port for connection
    pub fn host_port(&self) -> String {
        if (self.scheme == Scheme::Http && self.port == 80)
            || (self.scheme == Scheme::Https && self.port == 443)
        {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    /// Check if this URL uses TLS
    pub fn is_tls(&self) -> bool {
        self.scheme == Scheme::Https
    }

    /// Build a URL string
    pub fn to_string(&self) -> String {
        let mut s = format!("{}://{}", self.scheme.as_str(), self.host);

        if (self.scheme == Scheme::Http && self.port != 80)
            || (self.scheme == Scheme::Https && self.port != 443)
        {
            s.push_str(&format!(":{}", self.port));
        }

        s.push_str(&self.path);

        if let Some(q) = &self.query {
            s.push('?');
            s.push_str(q);
        }

        if let Some(f) = &self.fragment {
            s.push('#');
            s.push_str(f);
        }

        s
    }
}

/// URL-encode a string
pub fn encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            _ => {
                for b in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    result
}

/// URL-decode a string
pub fn decode(s: &str) -> CollectResult<String> {
    let mut result = Vec::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                let byte = u8::from_str_radix(&hex, 16).map_err(|_| {
                    CollectError::ParseError(format!("Invalid percent encoding: %{}", hex))
                })?;
                result.push(byte);
            } else {
                return Err(CollectError::ParseError(
                    "Incomplete percent encoding".into(),
                ));
            }
        } else if c == '+' {
            result.push(b' ');
        } else {
            result.push(c as u8);
        }
    }

    String::from_utf8(result).map_err(|e| CollectError::ParseError(format!("Invalid UTF-8: {}", e)))
}

/// Build a query string from key-value pairs
pub fn build_query(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_url() {
        let url = Url::parse("https://example.com/path").unwrap();
        assert_eq!(url.scheme, Scheme::Https);
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 443);
        assert_eq!(url.path, "/path");
        assert!(url.query.is_none());
    }

    #[test]
    fn test_parse_url_with_port() {
        let url = Url::parse("http://localhost:8080/api").unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host, "localhost");
        assert_eq!(url.port, 8080);
        assert_eq!(url.path, "/api");
    }

    #[test]
    fn test_parse_url_with_query() {
        let url = Url::parse("https://api.example.com/search?q=test&limit=10").unwrap();
        assert_eq!(url.path, "/search");
        assert_eq!(url.query, Some("q=test&limit=10".into()));
    }

    #[test]
    fn test_parse_url_with_fragment() {
        let url = Url::parse("https://example.com/page#section").unwrap();
        assert_eq!(url.path, "/page");
        assert_eq!(url.fragment, Some("section".into()));
    }

    #[test]
    fn test_request_path() {
        let url = Url::parse("https://example.com/api?key=value").unwrap();
        assert_eq!(url.request_path(), "/api?key=value");
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(encode("hello world"), "hello%20world");
        assert_eq!(encode("key=value"), "key%3Dvalue");
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(decode("hello%20world").unwrap(), "hello world");
        assert_eq!(decode("hello+world").unwrap(), "hello world");
    }

    #[test]
    fn test_build_query() {
        let query = build_query(&[("name", "John Doe"), ("age", "30")]);
        assert!(query.contains("name=John%20Doe"));
        assert!(query.contains("age=30"));
    }
}
