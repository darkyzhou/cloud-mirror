use cfg_if::cfg_if;
use normalize_url::normalizer::UrlNormalizer;
use worker::Url;

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
