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

impl<T: i18n::LocalizedDisplay> ReactiveLocalizedDisplay for &T {
    fn reactive_localize(&self) -> String {
        let langid = expect_langid();
        self.localize(&langid.get())
    }
}

/// The source of the langid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LangIdSource {
    Navigator,
    LocalStorage(String),
    Cookie(String),
}

/// Newtype wrapper around a langid signal used to pass it around via contexts.
#[derive(Debug, Clone, Copy)]
pub struct LangIdContext(RwSignal<i18n::LanguageIdentifier>);

/// A utility function for getting the langid signal.
pub fn use_langid() -> Option<RwSignal<i18n::LanguageIdentifier>> {
    use_context::<LangIdContext>().map(|ctx| ctx.0)
}

/// A utility function for getting the langid signal.
pub fn expect_langid() -> RwSignal<i18n::LanguageIdentifier> {
    use_langid().unwrap()
}

/// Returns a signal to reactively read and write a langid.
pub fn provide_langid_context(source: LangIdSource) {
    let langid = RwSignal::new({
        #[cfg(not(feature = "ssr"))]
        {
            let langid = window().navigator().language().unwrap_throw();
            let langid = i18n::LanguageIdentifier::from_str(&langid).unwrap_throw();
            langid
        }
    });

    provide_context(LangIdContext(langid));

    match source {
        LangIdSource::Navigator => {}
        LangIdSource::LocalStorage(key) => {
            #[cfg(not(feature = "ssr"))]
            {
                Effect::new({
                    let key = key.clone();
                    move |_| {
                        let langid = langid.read().to_string();
                        let storage_langid = crate::utils::local_storage::get(&key);

                        if let Some(storage_langid) = storage_langid {
                            if storage_langid != langid {
                                crate::utils::local_storage::set(&key, &langid);
                            }
                        }
                    }
                });

                set_timeout(
                    move || {
                        if let Some(item) = crate::utils::local_storage::get(&key) {
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
