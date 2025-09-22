//! `i18n-leptos` is a set of utilities for reactive localization in Leptos,
//! leveraging the `i18n` crate.
//!
//! This crate offers the `rtr!` macro, which simplifies the process of retrieving
//! localized messages from Fluent (FTL) files within a reactive Leptos context.
//! It integrates with Leptos's context system to automatically retrieve the current
//! language identifier and provides a reactive `Message` output.
//!
//! ## Features
//!
//! - **Reactive Translation**: Automatically re-evaluates translations when the
//!   language context changes.
//! - **Context-based Language**: Retrieves the `LanguageIdentifier` from a Leptos
//!   context, removing the need to explicitly pass it to the macro.
//! - **Fluent Integration**: Built on `i18n` for robust Fluent (FTL) message
//!   management, including arguments and attributes.
//! - **LocalizedDisplay Support**: Seamlessly integrates with types implementing
//!   `LocalizedDisplay` for reactive localization of complex objects.
//!
//! ## Usage
//!
//! To use this macro, you need to have `i18n` and `leptos` as dependencies
//! in your consuming crate, and ensure a `i18n_leptos::LangIdContext`
//! is provided in the Leptos context (e.g., via `i18n_leptos::provide_langid_context`).
//!
//! See the documentation for the `rtr!` macro for detailed usage examples.

pub use i18n;
pub use i18n_leptos_macros;

mod utils;

use std::str::FromStr;

use leptos::prelude::*;
use web_sys::wasm_bindgen::UnwrapThrowExt;

#[cfg(feature = "ssr")]
compile_error!("not implemented");

/// A reactive wrapper around `i18n::Message` that automatically re-evaluates
/// when the language context changes.
pub struct ReactiveMessage {
    msg: Signal<i18n::Message>,
}

impl ReactiveMessage {
    /// Creates a new `ReactiveMessage` from a signal function that produces an `i18n::Message`.
    pub fn new<F>(signal_fn: F) -> Self
    where
        F: Fn() -> i18n::Message + Send + Sync + 'static,
    {
        Self {
            msg: Signal::derive(signal_fn),
        }
    }

    /// Returns the ID of the localized message.
    ///
    /// This is a reactive read.
    pub fn id(&self) -> String {
        self.msg.read().id.clone()
    }

    /// Returns the ID of the localized message without tracking.
    pub fn id_untracked(&self) -> String {
        self.msg.read_untracked().id.clone()
    }

    /// Returns the translated value of the message.
    ///
    /// This is a reactive read.
    pub fn value(&self) -> String {
        self.msg.read().value.clone()
    }

    /// Returns the translated value of the message without tracking.
    pub fn value_untracked(&self) -> String {
        self.msg.read_untracked().value.clone()
    }

    /// Returns the value of a specific attribute of the message.
    /// If the attribute is not found, it returns the attribute name itself.
    ///
    /// This is a reactive read.
    pub fn attr(&self, attr: &str) -> String {
        self.msg
            .read()
            .attrs
            .get(attr)
            .cloned()
            .unwrap_or_else(move || attr.to_string())
    }

    /// Returns the value of a specific attribute of the message without tracking.
    /// If the attribute is not found, it returns the attribute name itself.
    pub fn attr_untracked(&self, attr: &str) -> String {
        self.msg
            .read_untracked()
            .attrs
            .get(attr)
            .cloned()
            .unwrap_or_else(move || attr.to_string())
    }
}

/// A trait for types that can be reactively localized.
pub trait ReactiveLocalizedDisplay {
    /// Localizes the implementor reactively, returning a `ReactiveMessage`.
    fn reactive_localize(self) -> ReactiveMessage;
}

impl<T> ReactiveLocalizedDisplay for T
where
    T: i18n::LocalizedDisplay + Send + Sync + 'static,
{
    fn reactive_localize(self) -> ReactiveMessage {
        ReactiveMessage::new(move || {
            let langid = expect_langid();
            self.localize(&langid.get())
        })
    }
}

/// Defines the source from which the `LanguageIdentifier` is obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LangIdSource {
    /// The language identifier is obtained from the browser's navigator language.
    Navigator,
    /// The language identifier is stored in and retrieved from local storage.
    LocalStorage(String),
    /// The language identifier is stored in and retrieved from a cookie.
    Cookie(String),
}

/// Newtype wrapper around a langid signal used to pass it around via contexts.
#[derive(Debug, Clone)]
struct LangIdContext(ArcRwSignal<i18n::LanguageIdentifier>);

/// A utility function for getting the langid signal from the Leptos context.
/// Returns `None` if no `LangIdContext` is provided.
pub fn use_langid() -> Option<ArcReadSignal<i18n::LanguageIdentifier>> {
    use_context::<LangIdContext>().map(|ctx| ctx.0.read_only())
}

/// A utility function for getting the langid signal from the Leptos context.
/// Panics if no `LangIdContext` is provided.
pub fn expect_langid() -> ArcReadSignal<i18n::LanguageIdentifier> {
    use_langid().unwrap()
}

const LANGID_EVENT_CHANGE_NAME: &'static str = "i18n-lang-change-notification";

/// Changes the current language identifier and dispatches a custom event to notify listeners.
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

/// Provides the `LangIdContext` to the Leptos context, initializing the language identifier
/// based on the specified `LangIdSource`.
///
/// This function sets up the reactive language identifier and handles its persistence
/// and updates based on the chosen source (Navigator, LocalStorage, or Cookie).
pub fn provide_langid_context(source: LangIdSource) {
    let initial_langid = {
        let langid = window().navigator().language().unwrap_throw();
        let langid = i18n::LanguageIdentifier::from_str(&langid).unwrap_throw();
        langid
    };
    let langid = ArcRwSignal::new(initial_langid.clone());

    provide_context(LangIdContext(langid.clone()));

    match source {
        LangIdSource::Navigator => {}
        LangIdSource::LocalStorage(key) => {
            // set initial local storage langid
            if let Some(storage_langid) = utils::local_storage::get(&key) {
                let new_langid = i18n::LanguageIdentifier::from_str(&storage_langid)
                    .unwrap_or(initial_langid.clone());
                langid.set(new_langid);
            }

            // handle programmatic change of theme
            let custom_event =
                leptos::ev::Custom::<leptos::ev::CustomEvent>::new(LANGID_EVENT_CHANGE_NAME);
            _ = leptos_use::use_event_listener(leptos_use::use_window(), custom_event, {
                let langid = langid.clone();
                let initial_langid = initial_langid.clone();
                let key = key.clone();
                move |data| {
                    let new_langid = data.detail().as_string().unwrap();
                    utils::local_storage::set(&key, &new_langid);
                    langid.set(
                        i18n::LanguageIdentifier::from_str(&new_langid)
                            .unwrap_or(initial_langid.clone()),
                    );
                }
            });

            // handle external modification of localStorage
            leptos_use::use_interval_fn(
                move || {
                    let langid = langid.clone();
                    let storage_langid = utils::local_storage::get(&key);
                    let now = langid.get_untracked();
                    if let Some(storage_langid) = storage_langid {
                        if storage_langid != now.to_string() {
                            if let Ok(new_langid) =
                                i18n::LanguageIdentifier::from_str(&storage_langid)
                            {
                                langid.set(new_langid);
                            }
                        }
                    }
                },
                1000,
            );
        }
        LangIdSource::Cookie(key) => {
            #[cfg(not(feature = "ssr"))]
            {
                Effect::new({
                    let langid = langid.clone();
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
