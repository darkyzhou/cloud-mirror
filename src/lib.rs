use rewriter::rewrite_html;
use url::ParseError;
use wasm_bindgen::JsValue;
use worker::*;

mod rewriter;
mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(mut req: Request, _env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);
    utils::set_panic_hook();

    let domain = _env
        .var("DOMAIN")
        .expect("Missing DOMAIN variable")
        .to_string();
    let proxy_url = Url::parse(domain.as_str()).expect("Invalid proxy url");
    let base_url = match req.url() {
        Err(err) => {
            return Response::error(format!("Unexpected url error: {}", err), 400);
        }
        Ok(req_url) => {
            if req_url.path() == "/" {
                return Response::redirect(
                    Url::parse("https://github.com/darkyzhou/cloudmirror").unwrap(),
                );
            }

            let url_to_visit = &req_url.path()[1..];
            match Url::parse(url_to_visit) {
                Err(ParseError::RelativeUrlWithoutBase) => {
                    let referer = req.headers().get("referer").ok();
                    match referer {
                        Some(Some(referer)) => {
                            let mut target_url = Url::parse(&format!(
                                "{}/{}",
                                referer.trim_end_matches('/'),
                                &req_url.path()[1..]
                            ))
                            .unwrap();
                            target_url.set_query(req_url.query());
                            target_url.set_fragment(req_url.fragment());
                            return Response::redirect(target_url);
                        }
                        _ => {
                            return Response::error("Invalid request url", 422);
                        }
                    }
                }
                Err(err @ _) => {
                    return Response::error(format!("Invalid request url: {}", err), 422);
                }
                Ok(mut url) => {
                    url.set_query(req_url.query());
                    url.set_fragment(req_url.fragment());
                    url
                }
            }
        }
    };

    let response = match req.method() {
        Method::Get => Fetch::Url(base_url.clone()).send().await,
        Method::Connect | Method::Trace => {
            return Response::error("The method is not supported by cloudmirror", 422);
        }
        _ => {
            let mut headers = req.headers().clone();
            _ = headers.delete("referer");

            let body = req.text().await.unwrap();
            let request = Request::new_with_init(
                base_url.as_str(),
                RequestInit::new()
                    .with_method(req.method().clone())
                    .with_redirect(RequestRedirect::Follow)
                    .with_body(Some(JsValue::from_str(&body)))
                    .with_headers(headers),
            )
            .expect("malformed Request object");
            Fetch::Request(request).send().await
        }
    };

    match response {
        Ok(mut response) => {
            match response.status_code() {
                200..=299 => {
                    // TODO: check if the body is too large to read
                    let is_html = response
                        .headers()
                        .get("content-type")
                        .map(|x| x.map(|x| x.starts_with("text/html"))) // TODO: check the charset
                        .map(|x| x.is_some() && x.unwrap())
                        .unwrap_or(false);
                    if !is_html {
                        _ = response
                            .headers_mut()
                            .set("Access-Control-Allow-Origin", proxy_url.as_str());
                        Ok(response)
                    } else {
                        match response.text().await {
                            Err(err) => {
                                console_error!(
                                    "Error requesting html {}, error: {}",
                                    req.path(),
                                    err
                                );
                                Response::error(
                                    format!("Error processing request to {}", req.path()),
                                    500,
                                )
                            }
                            Ok(html) => Response::from_html(rewrite_html(
                                &proxy_url,
                                &base_url,
                                html.as_str(),
                            )),
                        }
                    }
                }
                300..=399 => {
                    let location = response.headers().get("location");
                    if let Ok(Some(location)) = location {
                        response
                            .headers_mut()
                            .set(
                                "location",
                                &format!("{}/{}", proxy_url.to_string(), location),
                            )
                            .unwrap();
                    }
                    Ok(response)
                }
                _ => Ok(response),
            }
        }
        Err(err) => {
            console_error!("Error requesting {}, error: {}", req.path(), err);
            Response::error("Internal error", 500)
        }
    }
}
