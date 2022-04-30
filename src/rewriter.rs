use lol_html::{element, html_content::Element, HtmlRewriter, Settings};
use worker::{console_log, Url};

use crate::utils::{normalize_url, to_base_part};

pub fn rewrite_html(proxy_url: &Url, base_url: &Url, html: &str) -> String {
    let mut output = vec![];
    let mut rewriter = HtmlRewriter::new(
        Settings {
            element_content_handlers: vec![
                element!("a[href],link[rel=\"stylesheet\"]", |el| {
                    _ = rewrite_element_url(proxy_url, base_url, el, "href");
                    Ok(())
                }),
                element!("img[src],script[src],source[src]", |el| {
                    _ = rewrite_element_url(proxy_url, base_url, el, "src");
                    Ok(())
                }),
                element!("form[action]", |el| {
                    _ = rewrite_element_url(proxy_url, base_url, el, "action");
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
    proxy: &Url,
    base: &Url,
    element: &mut Element,
    attribute: &str,
) -> Result<(), ()> {
    let link = element.get_attribute(attribute);
    if let Some(link) = link {
        let rewrited = rewrite_url(proxy, base, link)?;
        _ = element.set_attribute(attribute, &rewrited);
    }
    Ok(())
}

pub fn rewrite_url(proxy: &Url, base: &Url, link: String) -> Result<String, ()> {
    // TODO: whitelist for global cdn, analytics, etc
    if link.trim().is_empty() || link.starts_with(proxy.as_str()) || link.starts_with("#") {
        return Ok(link);
    }

    if link.starts_with("//") {
        return Ok(format!("{}{}:{}", proxy.to_string(), base.scheme(), link));
    } else if link.starts_with("/") {
        return Ok(format!(
            "{}{}{}",
            proxy.to_string(),
            to_base_part(base),
            &link[1..]
        ));
    } else if !link.starts_with("http") {
        console_log!("got link: {}", &link);
        return Ok(format!(
            "{}{}{}",
            proxy.to_string(),
            base.to_string(),
            &link
        ));
    } else {
        Ok(format!("{}{}", proxy.to_string(), normalize_url(&link)?))
    }
}
