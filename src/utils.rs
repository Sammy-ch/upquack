use url::Url;

pub fn is_valid_url(url_str: &str)-> bool {
    match Url::parse(url_str) {
        Ok(url) => {
            let is_http_scheme = url.scheme() == "http" || url.scheme() == "https";
            let has_host = url.host().is_some();

           is_http_scheme && has_host 

        }
        Err(_) => {
            false
        }
    }
} 
