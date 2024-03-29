use tower_cookies::{Cookie, Cookies};

pub const JWT: &str = "jwt";

pub fn set_token_cookie(cookies: &Cookies, token: &str) {
    let mut cookie = Cookie::new(JWT, token.to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_secure(true);
    cookies.add(cookie);
}

pub fn remove_token_cookie(cookies: &Cookies) {
    // We add a new cookie with the same name, it will replace the previous one
    let mut cookie = Cookie::new(JWT, "");
    cookie.make_removal();
    cookies.add(cookie);
}
