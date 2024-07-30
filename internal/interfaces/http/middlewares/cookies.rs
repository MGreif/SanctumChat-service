use std::collections::HashMap;
use axum::{http::{Request, StatusCode, header::COOKIE}, middleware::Next, response::Response, body::Body};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Cookies {
    pub cookies: HashMap<String, String>,
}


pub async fn cookie_mw(mut request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let headers = request.headers();
    let cookies = if let Some(cookie_header) = headers.get(COOKIE) {
        // Parse the cookies from the header
        let cookies: HashMap<String, String> = cookie_header
            .to_str()
            .unwrap()
            .split("; ")
            .filter_map(|cookie_str| {
                let cookie = cookie::Cookie::parse(cookie_str).ok()?;
                Some((cookie.name().to_string(), cookie.value().to_string()))
            })
            .collect();

        Cookies { cookies }
    } else {
        Cookies {
            cookies: HashMap::new()
        }
    };

    request.extensions_mut().insert(cookies);
    let response: Response = next.run(request).await;
    Ok(response)

}