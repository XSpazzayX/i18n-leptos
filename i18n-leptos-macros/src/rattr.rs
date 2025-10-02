use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{Expr, LitStr, Token};

struct RattrMacroInput {
    msg: Expr,
    attr: LitStr,
    args: Vec<(LitStr, Expr)>,
}

impl Parse for RattrMacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let msg: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let attr: LitStr = input.parse()?;

        let mut args = Vec::new();
        while input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            let key: LitStr = input.parse()?;
            input.parse::<Token![=]>()?;
            let value: Expr = input.parse()?;
            args.push((key, value));
        }

        Ok(RattrMacroInput { msg, attr, args })
    }
}

pub fn rattr_impl(input: TokenStream) -> TokenStream {
    let RattrMacroInput { msg, attr, args } = match syn::parse(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error().into(),
    };

    if args.is_empty() {
        TokenStream::from(quote! { #msg.attr(#attr, None) })
    } else {
        let mut fluent_args = quote! { let mut args = i18n::FluentArgs::new(); };
        for (key, value) in args {
            fluent_args.extend(quote! { args.set(#key, #value); });
        }

        TokenStream::from(quote! {
            {
                #fluent_args
                #msg.attr(#attr, Some(&args))
            }
        })
    }
}
