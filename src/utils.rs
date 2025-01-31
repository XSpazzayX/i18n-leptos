use leptos::prelude::*;
use web_sys::wasm_bindgen::UnwrapThrowExt;

pub mod local_storage {
    use super::*;

    pub fn set(key: &str, value: &str) {
        window()
            .local_storage()
            .unwrap_throw()
            .unwrap_throw()
            .set_item(&key, &value)
            .unwrap_throw();
    }

    pub fn get(key: &str) -> Option<String> {
        window()
            .local_storage()
            .unwrap_throw()
            .unwrap_throw()
            .get_item(&key)
            .unwrap_throw()
    }
}

pub mod cookie {
    use super::*;
    use web_sys::wasm_bindgen::JsCast;

    pub fn set(key: &str, value: &str, attrs: &str) {
        let mut new_value = format!("{key}={value}");
        if !attrs.is_empty() {
            new_value.push_str("; ");
            new_value.push_str(attrs);
        }

        document()
            .dyn_into::<web_sys::HtmlDocument>()
            .unwrap()
            .set_cookie(&new_value)
            .unwrap_throw();
    }

    pub fn get(key: &str) -> Option<String> {
        let mut cookies = document()
            .dyn_into::<web_sys::HtmlDocument>()
            .unwrap()
            .cookie()
            .unwrap_or(String::default());
        cookies.insert_str(0, "; ");
        let result = cookies
            .split(&format!("; {key}="))
            .nth(1)
            .and_then(|cookie| cookie.split(';').next().map(String::from));

        result
    }
}
