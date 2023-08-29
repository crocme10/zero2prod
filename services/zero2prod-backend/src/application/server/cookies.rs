use tower_cookies::{Cookie, Cookies};

pub const JWT: &str = "jwt";

pub fn set_token_cookie(cookies: &Cookies, token: &str) {
    let mut cookie = Cookie::new(JWT, token.to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");

    cookies.add(cookie);
}

pub fn remove_token_cookie(cookies: &Cookies) {
    let mut cookie = Cookie::named(JWT);
    cookie.set_path("/");

    cookies.remove(cookie);
}
