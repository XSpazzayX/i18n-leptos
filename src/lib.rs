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
pub use i18n_leptos_macros::*;

mod ctx;
mod utils;

pub use ctx::*;

use leptos::prelude::*;

#[cfg(feature = "ssr")]
compile_error!("not implemented");

/// A reactive wrapper around `i18n::Message` that automatically re-evaluates
/// when the language context changes.
#[derive(Clone, Copy)]
pub struct ReactiveMessage {
    msg: RwSignal<i18n::Message>,
}

impl ReactiveMessage {
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
    pub fn attr(&self, attr: &str, args: Option<&i18n::FluentArgs>) -> String {
        self.msg.track();
        self.msg
            .write_untracked()
            .attrs
            .get_mut(attr)
            .map(|attr_cache| match attr_cache.query(args) {
                Ok(value) => value,
                Err(err) => {
                    log::error!(
                        "i18n_leptos | an error occurred during localization of '{attr}': {err:?}"
                    );
                    attr.to_string()
                }
            })
            .unwrap_or_else(move || attr.to_string())
    }

    /// Returns the value of a specific attribute of the message without tracking.
    /// If the attribute is not found, it returns the attribute name itself.
    pub fn attr_untracked(&self, attr: &str, args: Option<&i18n::FluentArgs>) -> String {
        self.msg
            .write_untracked()
            .attrs
            .get_mut(attr)
            .map(|attr_cache| match attr_cache.query(args) {
                Ok(value) => value,
                Err(err) => {
                    log::error!(
                        "i18n_leptos | an error occurred during localization of '{attr}': {err:?}"
                    );
                    attr.to_string()
                }
            })
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
        let msg = RwSignal::default();

        Effect::new(move || {
            let langid = ctx::expect_langid();
            msg.set(self.localize(&langid.get()));
        });

        ReactiveMessage { msg }
    }
}
