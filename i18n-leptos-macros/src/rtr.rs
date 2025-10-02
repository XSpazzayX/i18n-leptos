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

enum RtrArg {
    Locales(Ident),
    Main {
        key: Ident,
        value: Expr,
    },
    Attribute {
        attr: LitStr,
        key: Ident,
        value: Expr,
    },
}

struct RtrArgs {
    locales_var: Ident,
    main_args: Vec<(Ident, Expr)>,
    attr_args: HashMap<String, Vec<(Ident, Expr)>>,
}

impl Parse for RtrArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut locales_var = Ident::new("LOCALES", Span::call_site());
        let mut main_args = Vec::new();
        let mut attr_args: HashMap<String, Vec<(Ident, Expr)>> = HashMap::new();

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let arg = input.parse::<RtrArg>()?;
            match arg {
                RtrArg::Locales(ident) => locales_var = ident,
                RtrArg::Main { key, value } => main_args.push((key, value)),
                RtrArg::Attribute { attr, key, value } => {
                    attr_args
                        .entry(attr.value())
                        .or_default()
                        .push((key, value));
                }
            }
        }

        Ok(RtrArgs {
            locales_var,
            main_args,
            attr_args,
        })
    }
}

impl Parse for RtrArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Ident) && input.peek2(Token![=]) {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if key == "locales" {
                Ok(RtrArg::Locales(input.parse()?))
            } else {
                Ok(RtrArg::Main {
                    key,
                    value: input.parse()?,
                })
            }
        } else if lookahead.peek(Ident) && input.peek2(syn::token::Paren) {
            let _attr_ident: Ident = input.parse()?; // Parse 'attr'
            let content;
            syn::parenthesized!(content in input); // Parse content within parentheses

            let attr_id: LitStr = content.parse()?;
            content.parse::<Token![,]>()?;
            let arg_key: Ident = content.parse()?;
            content.parse::<Token![=]>()?;
            let arg_value: Expr = content.parse()?;

            Ok(RtrArg::Attribute {
                attr: attr_id,
                key: arg_key,
                value: arg_value,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

struct RtrMacroInput {
    kind: RtrInputKind,
    args: RtrArgs,
}

impl Parse for RtrMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        let kind = if lookahead.peek(LitStr) {
            RtrInputKind::MessageId(input.parse()?)
        } else {
            RtrInputKind::LocalizedDisplayExpr(input.parse()?)
        };

        let args = input.parse::<RtrArgs>()?;

        Ok(RtrMacroInput { kind, args })
    }
}

pub fn rtr_impl(input: TokenStream) -> TokenStream {
    let RtrMacroInput { kind, args } = match syn::parse(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };

    match kind {
        RtrInputKind::MessageId(id) => {
            let RtrArgs {
                locales_var,
                main_args,
                attr_args,
            } = args;
            let mut query_builder = quote! { i18n::Query::new(#id) };

            let main_args_tokens: Vec<_> = main_args
                .into_iter()
                .map(|(key, value)| quote! { .with_arg(stringify!(#key), #value) })
                .collect();

            let attr_args_tokens: Vec<_> = attr_args
                .into_iter()
                .flat_map(|(attr_name, args)| {
                    args.into_iter().map(move |(key, value)| {
                        quote! { .with_attr_arg(#attr_name, stringify!(#key), #value) }
                    })
                })
                .collect();

            query_builder.extend(main_args_tokens);
            query_builder.extend(attr_args_tokens);

            let query_call_block = quote! {
                #locales_var.query(&langid.get(), &#query_builder)
            };

            let final_expansion = quote! {
                {
                    let msg = leptos::prelude::RwSignal::default();

                    leptos::prelude::Effect::new(move || {
                        let langid = i18n_leptos::expect_langid();
                        msg.set(#query_call_block.unwrap_or_else(|_errs| {
                            i18n::Message {
                                id: #id.to_string(),
                                value: #id.to_string(),
                                attrs: Default::default(),
                            }
                        }));
                    });

                    i18n_leptos::ReactiveMessage { msg }
                }
            };
            TokenStream::from(final_expansion)
        }
        RtrInputKind::LocalizedDisplayExpr(expr) => {
            if !args.main_args.is_empty() || !args.attr_args.is_empty() {
                let mut error = syn::Error::new_spanned(
                    expr,
                    "Arguments are not supported when passing a LocalizedDisplay object.",
                );
                if !args.main_args.is_empty() {
                    for (key, _) in args.main_args {
                        error.combine(syn::Error::new_spanned(
                            key,
                            "Main arguments are not supported here.",
                        ));
                    }
                }
                if !args.attr_args.is_empty() {
                    for (attr, args) in args.attr_args {
                        for (key, _) in args {
                            error.combine(syn::Error::new_spanned(
                                key,
                                format!(
                                    "Attribute arguments for '{}' are not supported here.",
                                    attr
                                ),
                            ));
                        }
                    }
                }
                return error.to_compile_error().into();
            }
            TokenStream::from(quote! { #expr.reactive_localize() })
        }
    }
}
