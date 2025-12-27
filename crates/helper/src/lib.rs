#![no_std]
use aidoku::{
    AidokuError, Result,
    alloc::string::{String, ToString},
    imports::defaults::{DefaultValue, defaults_get, defaults_set},
};
pub const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:142.0) Gecko/20100101 Firefox/142.0";

const BASE_URL_KEY: &str = "baseUrl";

pub fn get_base_url() -> Result<String> {
    defaults_get::<String>(BASE_URL_KEY)
        .ok_or_else(|| AidokuError::Message("Unknown Key".to_string()))
}

pub fn set_base_url(base_url: &str) {
    defaults_set(BASE_URL_KEY, DefaultValue::String(base_url.to_string()));
}
