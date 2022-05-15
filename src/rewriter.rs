use lol_html::{element, html_content::Element, HtmlRewriter, Settings};
use worker::Url;

use crate::utils::{get_base_part, normalize_url};

pub fn rewrite_html(proxy_url: &Url, origin_url: &Url, html: &str) -> String {
    let mut output = vec![];
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("a[href],link[rel=\"stylesheet\"]", |el| {
                    _ = rewrite_element_url(proxy_url, origin_url, el, "href");
                    Ok(())
                }),
                element!("img[src],script[src],source[src]", |el| {
                    _ = rewrite_element_url(proxy_url, origin_url, el, "src");
                    Ok(())
                }),
                element!("form[action]", |el| {
                    _ = rewrite_element_url(proxy_url, origin_url, el, "action");
                    Ok(())
                }),
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c),
    );
    rewriter.write(html.as_bytes()).unwrap();
    rewriter.end().unwrap();
    String::from_utf8(output).unwrap()
}

pub fn rewrite_element_url(
    proxy_url: &Url,
    origin_url: &Url,
    element: &mut Element,
    attribute: &str,
) -> Result<(), ()> {
    let link = element.get_attribute(attribute);
    if let Some(link) = link {
        let rewrited = rewrite_url(proxy_url, origin_url, link)?;
        _ = element.set_attribute(attribute, &rewrited);
    }
    Ok(())
}

pub fn rewrite_url(proxy_url: &Url, origin_url: &Url, link: String) -> Result<String, ()> {
    // TODO: whitelist for global cdn, analytics, etc
    if link.trim().is_empty()
        || link.starts_with(proxy_url.as_str())
        || link.starts_with("#")
        || link.starts_with("/")
        || !link.starts_with("http")
    {
        return Ok(link);
    }

    if link.starts_with("//") {
        Ok(format!(
            "{}{}:{}",
            proxy_url.as_str(),
            origin_url.scheme(),
            link
        ))
    } else if link.starts_with(get_base_part(origin_url).as_str()) {
        let link_url = Url::parse(link.as_str()).map_err(|_| ())?;
        let mut url = proxy_url.clone();
        url.set_path(link_url.path());
        url.set_query(link_url.query());
        url.set_fragment(link_url.fragment());
        Ok(url.to_string())
    } else {
        Ok(format!("{}{}", proxy_url.as_str(), normalize_url(&link)?))
    }
}
