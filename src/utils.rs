use cfg_if::cfg_if;
use normalize_url::normalizer::UrlNormalizer;
use worker::{Headers, Url};

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn to_base_part(url: &Url) -> String {
    let mut clone = url.clone();
    clone.set_path("/");
    clone.set_query(None);
    clone.set_fragment(None);
    clone.to_string()
}

pub fn normalize_url(url: &str) -> Result<String, ()> {
    UrlNormalizer::new(url)
        .map_err(|_| ())?
        .normalize(None)
        .map_err(|_| ())
}

pub fn clean_headers(headers: &mut Headers, base_url: &Url) -> Result<(), ()> {
    headers.delete("referer").map_err(|_| ())?;
    headers
        .set("host", base_url.host_str().expect("malformed request url"))
        .map_err(|_| ())?;
    let targets: Vec<String> = headers
        .keys()
        .filter(|name| name.starts_with("cf-") || name.starts_with("x-"))
        .collect();
    for name in targets {
        headers.delete(name.as_str()).map_err(|_| ())?;
    }
    Ok(())
}
