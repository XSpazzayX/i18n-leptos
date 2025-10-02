use crate::utils;
use leptos::prelude::*;
use std::str::FromStr;
use web_sys::wasm_bindgen::UnwrapThrowExt;

/// Defines the source from which the `LanguageIdentifier` is obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LangIdSource {
    /// The language identifier is obtained from the browser's navigator language.
    Navigator,
    /// The language identifier is stored in and retrieved from local storage.
    LocalStorage(String),
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

/// The custom event name.
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
pub fn provide_langid_context(
    source: LangIdSource,
    initial_langid: Option<i18n::LanguageIdentifier>,
) {
    let initial_langid = match initial_langid {
        Some(langid) => langid,
        None => {
            let langid = window()
                .navigator()
                .language()
                .unwrap_or_else(move || "en-US".to_string());
            let langid = i18n::LanguageIdentifier::from_str(&langid).unwrap_throw();
            langid
        }
    };
    let langid = ArcRwSignal::new(initial_langid.clone());

    provide_context(LangIdContext(langid.clone()));

    match source {
        LangIdSource::Navigator => {}
        LangIdSource::LocalStorage(key) => {
            setup_local_storage_handler(langid, initial_langid, key);
        }
    }
}

fn setup_local_storage_handler(
    langid: ArcRwSignal<i18n::LanguageIdentifier>,
    initial_langid: i18n::LanguageIdentifier,
    key: String,
) {
    // set initial local storage langid
    if let Ok(Some(storage_langid)) = utils::local_storage::get(&key) {
        let new_langid =
            i18n::LanguageIdentifier::from_str(&storage_langid).unwrap_or(initial_langid.clone());
        langid.set(new_langid);
    }

    // handle programmatic change of theme
    let custom_event = leptos::ev::Custom::<leptos::ev::CustomEvent>::new(LANGID_EVENT_CHANGE_NAME);
    _ = leptos_use::use_event_listener(leptos_use::use_window(), custom_event, {
        let langid = langid.clone();
        let initial_langid = initial_langid.clone();
        let key = key.clone();
        move |data| {
            let new_langid = match data.detail().as_string() {
                Some(langid) => langid,
                None => {
                    log::error!("invalid data passed in the '{LANGID_EVENT_CHANGE_NAME}' event");
                    return;
                }
            };
            if let Err(err) = utils::local_storage::set(&key, &new_langid) {
                log::error!("failed to set langid in local storage: {err:?}");
            }
            langid.set(
                i18n::LanguageIdentifier::from_str(&new_langid).unwrap_or(initial_langid.clone()),
            );
        }
    });
}
