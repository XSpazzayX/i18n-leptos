//! `i18n-leptos-macros` provides procedural macros for reactive internationalization
//! in Leptos applications, leveraging the `i18n` crate.
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
//! To use this macro, you need to have `leptos` as a dependency
//! in your consuming crate.
//!
//! See the documentation for the `rtr!` macro for detailed usage examples.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Expr, Ident, LitStr, Token};

enum RtrInputKind {
    MessageId(LitStr),
    LocalizedDisplayExpr(Expr),
}

struct RtrMacroInput {
    kind: RtrInputKind,
    locales_var: Ident,
    main_args: Vec<(Ident, Expr)>,
    attr_args: HashMap<String, Vec<(Ident, Expr)>>,
    on_error: Option<Expr>,
}

impl Parse for RtrMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let kind = if lookahead.peek(LitStr) {
            RtrInputKind::MessageId(input.parse()?)
        } else if lookahead.peek(Ident)
            || lookahead.peek(syn::token::Paren)
            || lookahead.peek(syn::token::Bracket)
            || lookahead.peek(syn::token::Brace)
            || lookahead.peek(Token![<])
            || lookahead.peek(Token![&])
            || lookahead.peek(Token![*])
            || lookahead.peek(Token![!])
            || lookahead.peek(Token![~])
            || lookahead.peek(Token![+])
            || lookahead.peek(Token![-])
            || lookahead.peek(Token![.])
            || lookahead.peek(Token![::])
        {
            RtrInputKind::LocalizedDisplayExpr(input.parse()?)
        } else {
            return Err(lookahead.error());
        };

        let mut locales_var = Ident::new("LOCALES", Span::call_site());
        let mut main_args = Vec::new();
        let mut attr_args: HashMap<String, Vec<(Ident, Expr)>> = HashMap::new();
        let mut on_error = None;

        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            if input.peek(Ident) && input.peek2(Token![=]) {
                let key_ident: Ident = input.parse()?;
                input.parse::<Token![=]>()?;

                if key_ident == "locales" {
                    locales_var = input.parse()?;
                } else if key_ident == "on_error" {
                    on_error = Some(input.parse()?);
                } else {
                    main_args.push((key_ident, input.parse()?));
                }
            } else if input.peek(Ident) && input.peek2(syn::token::Paren) {
                let _attr_ident: Ident = input.parse()?; // Parse 'attr'
                let content;
                syn::parenthesized!(content in input); // Parse content within parentheses

                let attr_id: LitStr = content.parse()?;
                content.parse::<Token![,]>()?;
                let arg_key: Ident = content.parse()?;
                content.parse::<Token![=]>()?;
                let arg_value: Expr = content.parse()?;

                attr_args
                    .entry(attr_id.value())
                    .or_default()
                    .push((arg_key, arg_value));
            } else {
                return Err(input.error("Unexpected token. Expected `locales = VAR_NAME`, `on_error = EXPR`, or message arguments."));
            }
        }

        Ok(RtrMacroInput {
            kind,
            locales_var,
            main_args,
            attr_args,
            on_error,
        })
    }
}

/// A reactive procedural macro for internationalization in Leptos applications.
///
/// The `rtr!` macro provides a convenient way to retrieve localized messages
/// from Fluent (FTL) files, automatically reacting to changes in the language
/// context provided by Leptos.
///
/// It supports two primary modes of operation:
///
/// 1.  **Message ID Lookup**: Translates a message ID (string literal) using the
///     current language from the Leptos context. This mode supports arguments
///     and custom error handling.
/// 2.  **LocalizedDisplay Object**: Calls the `.reactive_localize()` method on an
///     object that implements the `LocalizedDisplay` trait. This mode does not
///     support additional arguments or error handling within the macro itself,
///     as the `LocalizedDisplay` implementation is expected to handle its own
///     localization logic.
///
/// Both modes return a `i18n_leptos::ReactiveMessage`, ensuring that
/// your UI automatically updates when the language changes.
///
/// ## Language Context
///
/// The macro expects a `i18n_leptos::LangIdContext` to be available in the Leptos context.
/// This can be provided using `i18n_leptos::provide_langid_context`.
///
/// ## Syntax
///
/// ```rust
/// // Mode 1: Message ID Lookup
/// rtr!("message-id" [, locales = VAR_NAME] [, key = value]* [, attr("attr-id", key = value)* ] [, on_error = EXPR]);
///
/// // Mode 2: LocalizedDisplay Object
/// rtr!(localized_object_expr);
/// ```
///
/// ### Parameters
///
/// -   **`"message-id"`**: A string literal representing the ID of the Fluent message to translate.
///
/// -   **`localized_object_expr`**: An expression that evaluates to an object implementing
///     the `LocalizedDisplay` trait. When this is used, no other parameters are allowed.
///
/// -   **`locales = VAR_NAME`** (optional, Mode 1 only): An identifier for the
///     `i18n::Locales` static variable to use. Defaults to `LOCALES`.
///
/// -   **`key = value`** (optional, Mode 1 only): Key-value pairs for arguments to the
///     main message. `key` must be an identifier, and `value` can be any Rust expression.
///
/// -   **`attr("attr-id", key = value)`** (optional, Mode 1 only): Arguments for a
///     specific attribute of the message. `"attr-id"` is a string literal representing
///     the attribute ID. `key` must be an identifier, and `value` can be any Rust expression.
///
/// -   **`on_error = EXPR`** (optional, Mode 1 only): A Rust expression (e.g., a closure
///     or function call) that will be executed if the localization query fails.
///     It must accept one argument of type `Vec<i18n::FluentError>` and return
///     an `i18n::Message`. If not provided, a default `i18n::Message`
///     is returned (with the message ID as its value).
///
/// ## Returns
///
/// A `i18n_leptos::ReactiveMessage`.
#[proc_macro]
pub fn rtr(input: TokenStream) -> TokenStream {
    let RtrMacroInput {
        kind,
        locales_var,
        on_error,
        main_args,
        attr_args,
    } = match syn::parse(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };

    match kind {
        RtrInputKind::MessageId(id) => {
            let mut query_builder = quote! { i18n::Query::new(#id) };

            for (key, value) in main_args.into_iter() {
                query_builder = quote! { #query_builder.with_arg(stringify!(#key), #value) };
            }
            for (attr_name, args) in attr_args.into_iter() {
                for (key, value) in args {
                    query_builder = quote! { #query_builder.with_attr_arg(#attr_name, stringify!(#key), #value) };
                }
            }

            let query_call_block = quote! {
                let langid = leptos::use_context::<i18n_leptos::LangIdContext>().expect("LangIdContext not found. Make sure you've provided it.");
                #locales_var.query(&langid.get(), &#query_builder)
            };

            let error_handling_block = if let Some(on_error_expr) = on_error {
                quote! {
                    match { #query_call_block } {
                        Ok(msg) => msg,
                        Err(errs) => (#on_error_expr)(errs),
                    }
                }
            } else {
                quote! {
                    match { #query_call_block } {
                        Ok(msg) => msg,
                        Err(errs) => {
                            i18n::Message {
                                id: #id.to_string(),
                                value: #id.to_string(),
                                attrs: std::collections::HashMap::new(),
                            }
                        }
                    }
                }
            };

            let final_expansion = quote! {
                i18n_leptos::ReactiveMessage::new(move || {
                    #error_handling_block
                })
            };
            TokenStream::from(final_expansion)
        }
        RtrInputKind::LocalizedDisplayExpr(expr) => {
            if on_error.is_some() || !main_args.is_empty() || !attr_args.is_empty() {
                return syn::Error::new_spanned(
                    expr,
                    "Arguments (on_error, key=value, attr(...)) are not supported when passing a LocalizedDisplay object.",
                )
                .to_compile_error()
                .into();
            }
            TokenStream::from(quote! { #expr.reactive_localize() })
        }
    }
}
