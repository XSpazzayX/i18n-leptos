//! A set of utilities for reactive localization in Leptos.

mod utils;

use std::str::FromStr;

use leptos::prelude::*;
use web_sys::wasm_bindgen::UnwrapThrowExt;

#[cfg(feature = "ssr")]
compile_error!("not implemented");

/// Wraps the localizing type in a signal.
#[macro_export]
macro_rules! tr {
    ($e:expr) => {
        Signal::derive(move || $e.reactive_localize())
    };
}

/// Reactively localizes the type given the currently set langid.
pub trait ReactiveLocalizedDisplay {
    /// Reactively localizes the type.
    ///
    /// # Safety
    /// Must not be called outside of a reactive context since it calls `use_context()`.
    fn reactive_localize(&self) -> String;
}

impl<T: i18n::LocalizedDisplay> ReactiveLocalizedDisplay for T {
    fn reactive_localize(&self) -> String {
        let langid = expect_langid();
        self.localize(&langid.get())
    }
}

/// The source of the langid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LangIdSource {
    Navigator,
    LocalStorage,
    Cookie(String),
}

/// Newtype wrapper around a langid signal used to pass it around via contexts.
#[derive(Debug, Clone, Copy)]
struct LangIdContext(RwSignal<i18n::LanguageIdentifier>);

/// A utility function for getting the langid signal.
pub fn use_langid() -> Option<ReadSignal<i18n::LanguageIdentifier>> {
    use_context::<LangIdContext>().map(|ctx| ctx.0.read_only())
}

/// A utility function for getting the langid signal.
pub fn expect_langid() -> ReadSignal<i18n::LanguageIdentifier> {
    use_langid().unwrap()
}

const KEY_NAME: &'static str = "i18n-lang";
const LANGID_EVENT_CHANGE_NAME: &'static str = "i18n-lang-change-notification";

/// Changes the langid given browser preference or explicit selection.
pub fn change_langid(langid: i18n::LanguageIdentifier) {
    let langid = langid.to_string();
    let custom_event_init = web_sys::CustomEventInit::new();
    custom_event_init.set_detail(&langid.into());
    let custom_event = leptos::ev::CustomEvent::new_with_event_init_dict(
        LANGID_EVENT_CHANGE_NAME,
        &custom_event_init,
    )
    .expect("should pass always");
    _ = window().dispatch_event(&custom_event);
}

/// Returns a signal to reactively read and write a langid.
pub fn provide_langid_context(source: LangIdSource) {
    let initial_langid = {
        let langid = window().navigator().language().unwrap_throw();
        let langid = i18n::LanguageIdentifier::from_str(&langid).unwrap_throw();
        langid
    };
    let langid = RwSignal::new(initial_langid.clone());

    provide_context(LangIdContext(langid));

    match source {
        LangIdSource::Navigator => {}
        LangIdSource::LocalStorage => {
            // handle programmatic change of theme
            let custom_event =
                leptos::ev::Custom::<leptos::ev::CustomEvent>::new(LANGID_EVENT_CHANGE_NAME);
            _ = leptos_use::use_event_listener(leptos_use::use_window(), custom_event, {
                let initial_langid = initial_langid.clone();
                move |data| {
                    let new_langid = data.detail().as_string().unwrap();
                    utils::local_storage::set(KEY_NAME, &new_langid);
                    langid.set(
                        i18n::LanguageIdentifier::from_str(&new_langid)
                            .unwrap_or(initial_langid.clone()),
                    );
                }
            });

            // handle external modification of localStorage
            _ = leptos_use::use_event_listener(leptos_use::use_window(), leptos::ev::storage, {
                let initial_langid = initial_langid.clone();
                move |ev| match ev.key() {
                    Some(key) => {
                        if key == KEY_NAME {
                            match ev.new_value() {
                                Some(new_langid) => {
                                    let new_langid =
                                        i18n::LanguageIdentifier::from_str(&new_langid)
                                            .unwrap_or(initial_langid.clone());
                                    langid.set(new_langid);
                                }
                                None => langid.set(initial_langid.clone()),
                            }
                        }
                    }
                    None => langid.set(initial_langid.clone()),
                }
            });
        }
        LangIdSource::Cookie(key) => {
            #[cfg(not(feature = "ssr"))]
            {
                Effect::new({
                    let key = key.clone();
                    move |_| {
                        let langid = langid.read().to_string();
                        let cookie_langid = crate::utils::cookie::get(&key);

                        if let Some(cookie_langid) = cookie_langid {
                            if cookie_langid != langid {
                                crate::utils::cookie::set(&key, &langid, "");
                            }
                        } else {
                            crate::utils::cookie::set(&key, &langid, "");
                        }
                    }
                });

                set_timeout(
                    move || {
                        if let Some(item) = crate::utils::cookie::get(&key) {
                            let langid_str = langid.read_untracked().to_string();
                            if item != langid_str {
                                if let Ok(item) = i18n::LanguageIdentifier::from_str(&item) {
                                    langid.set(item);
                                }
                            }
                        }
                    },
                    std::time::Duration::from_secs(1),
                );
            }
        }
    }
}
