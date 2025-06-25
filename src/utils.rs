use url::Url;

pub fn is_valid_url(url_str: &str) -> bool {
    match Url::parse(url_str) {
        Ok(url) => {
            let is_http_scheme = url.scheme() == "http" || url.scheme() == "https";
            let has_host = url.host().is_some();

            let has_valid_tld = if let Some(host) = url.host_str() {
                let valid_tlds = [
                    ".com", ".org", ".net", ".io", ".co", ".gov", ".edu", ".dev", "bi",
                ];
                valid_tlds.iter().any(|tld| host.ends_with(tld))
            } else {
                false
            };

            is_http_scheme && has_host && has_valid_tld
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_invalid_url() {
        let invalid_url = [
            "htt://example.com",      // Invalid scheme
            "https://127.0.0.1",      // IP address, not a domain with TLD
            "mailto:rms@example.net", // Invalid scheme
            "https://example",        // Missing TLD
            "https://example.xyz",    // TLD not in our valid list
            "http://localhost",       // localhost, no TLD
        ];

        for url in invalid_url {
            assert_eq!(is_valid_url(url), false, "Expected false for: {}", url);
        }
    }

    #[test]
    fn parse_valid_url() {
        let valid_url = [
            "https://www.example.com",
            "http://subdomain.example.org",
            "https://anothersite.net",
            "http://my-app.io",
            "https://docs.google.com",
            "http://example.dev",
        ];

        for url in valid_url {
            assert_eq!(is_valid_url(url), true, "Expected true for: {}", url);
        }
    }
}
