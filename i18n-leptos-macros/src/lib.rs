use proc_macro::TokenStream;

mod rattr;
mod rtr;

/// A reactive procedural macro for internationalization in Leptos applications.
///
/// The `rtr!` macro provides a convenient way to retrieve localized messages
/// from Fluent (FTL) files, automatically reacting to changes in the language
/// context provided by Leptos.
///
/// It supports two primary modes of operation:
///
/// 1.  **Message ID Lookup**: Translates a message ID (string literal) using the
///     current language from the Leptos context. This mode supports arguments.
/// 2.  **LocalizedDisplay Object**: Calls the `.reactive_localize()` method on an
///     object that implements the `LocalizedDisplay` trait. This mode does not
///     support additional arguments within the macro itself,
///     as the `LocalizedDisplay` implementation is expected to handle its own
///     localization logic.
///
/// Both modes return a `i18n_leptos::ReactiveMessage`, ensuring that
/// your UI automatically updates when the language changes.
///
/// ## Language Context
/// The macro expects a `i18n_leptos::LangIdContext` to be available in the Leptos context.
/// This can be provided using `i18n_leptos::provide_langid_context`.
///
/// ## Syntax
/// ```ignore
/// // Mode 1: Message ID Lookup
/// rtr!("message-id" [, locales = VAR_NAME] [, key = value]* [, attr("attr-id", key = value)* ]);
///
/// // Mode 2: LocalizedDisplay Object
/// rtr!(localized_object_expr);
/// ```
///
/// ### Parameters
/// -   **`"message-id"`**: A string literal representing the ID of the Fluent message to translate.
/// -   **`localized_object_expr`**: An expression that evaluates to an object implementing
///     the `LocalizedDisplay` trait. When this is used, no other parameters are allowed.
/// -   **`locales = VAR_NAME`** (optional, Mode 1 only): An identifier for the
///     `i18n::Locales` static variable to use. Defaults to `LOCALES`.
/// -   **`key = value`** (optional, Mode 1 only): Key-value pairs for arguments to the
///     main message. `key` must be an identifier, and `value` can be any Rust expression.
/// -   **`attr("attr-id", key = value)`** (optional, Mode 1 only): Arguments for a
///     specific attribute of the message. `"attr-id"` is a string literal representing
///     the attribute ID. `key` must be an identifier, and `value` can be any Rust expression.
///
/// ## Returns
/// A `i18n_leptos::ReactiveMessage`.
#[proc_macro]
pub fn rtr(input: TokenStream) -> TokenStream {
    rtr::rtr_impl(input)
}

/// A macro to reactively get an attribute from a `ReactiveMessage`.
///
/// This macro simplifies the process of retrieving an attribute from a `ReactiveMessage`,
/// and supports passing arguments to the attribute.
///
/// ## Syntax
/// ```ignore
/// rattr!(reactive_message, "attribute-name" [, key = value]*);
/// ```
///
/// ### Parameters
/// -   **`reactive_message`**: An expression that evaluates to a `ReactiveMessage`.
/// -   **`"attribute-name"`**: A string literal representing the name of the attribute to retrieve.
/// -   **`key = value`** (optional): Key-value pairs for arguments to the attribute.
///     `key` must be an identifier, and `value` can be any Rust expression.
///
/// ## Returns
/// A `String` representing the value of the attribute.
#[proc_macro]
pub fn rattr(input: TokenStream) -> TokenStream {
    rattr::rattr_impl(input)
}
