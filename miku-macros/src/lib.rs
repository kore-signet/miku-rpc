use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    bracketed, parse_macro_input,
    token::{self, Comma},
    FnArg, Ident, LitStr, Pat, Path, Result, ReturnType, Token, Type, Generics
};

struct MethodData {
    rust_name: Ident,
    ret_type: Box<Type>,
    generics: Generics,
    method_name: LitStr, // oc2 method
    args: Punctuated<FnArg, Comma>,
}

impl Parse for MethodData {
    fn parse(input: ParseStream) -> Result<Self> {
        let rust_name = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;

        let ret_type = if let ReturnType::Type(_, t) = input.parse::<ReturnType>()? {
            t
        } else {
            return Err(input.error("return type needs to be specified"));
        };


        input.parse::<Token![;]>()?;
        let method_name = input.parse::<LitStr>()?;

        let args = if input.is_empty() {
            Punctuated::new()
        } else {
            input.parse::<Token![;]>()?;
            input.parse_terminated(FnArg::parse)?
        };

        Ok(MethodData {
            method_name,
            rust_name,
            ret_type,
            generics,
            args,
        })
    }
}

#[proc_macro]
pub fn rpc(tokens: TokenStream) -> TokenStream {
    let MethodData {
        method_name,
        rust_name,
        generics,
        ret_type,
        args,
        ..
    } = parse_macro_input!(tokens as MethodData);

    let arg_idents: Vec<Box<Pat>> = args
        .iter()
        .filter_map(|v| {
            if let FnArg::Typed(t) = v {
                Some(t.pat.clone())
            } else {
                None
            }
        })
        .collect();
    let arg_defs = args.into_iter();

    let tokens = quote! {
        fn #rust_name #generics (&self, bus: &mut  crate::bus::DeviceBus, #(#arg_defs),*) -> std::io::Result<#ret_type> {
            let response: crate::Response<#ret_type> = bus.call(&crate::Call::invoke(self.id(), #method_name, &[#(&#arg_idents),*]))?;
            Ok(response.data)
        }
    };

    TokenStream::from(tokens)
}

struct DeviceData {
    rust_name: Ident,
    oc2_identity: LitStr,
    _bracket_token: token::Bracket,
    capabilities: Punctuated<Path, Comma>,
}

impl Parse for DeviceData {
    fn parse(input: ParseStream) -> Result<Self> {
        let rust_name = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let oc2_identity = input.parse::<LitStr>()?;
        input.parse::<Comma>()?;

        let content;
        Ok(DeviceData {
            rust_name,
            oc2_identity,
            _bracket_token: bracketed!(content in input),
            capabilities: content.parse_terminated(Path::parse)?,
        })
    }
}

#[proc_macro]
pub fn define_device(tokens: TokenStream) -> TokenStream {
    let DeviceData {
        rust_name,
        oc2_identity,
        capabilities,
        ..
    } = parse_macro_input!(tokens as DeviceData);

    let capabilities = capabilities.into_iter();

    let tokens = quote! {
        pub struct #rust_name(pub String);

        impl RPCDevice for #rust_name {
            fn id(&self) -> &str {
                &self.0
            }

            fn from_id(id: String) -> Self {
                #rust_name(id)
            }
        }

        impl IdentifiedDevice for #rust_name {
            const IDENTITY: &'static str = #oc2_identity;
        }

        #(impl #capabilities for #rust_name {})*
    };

    TokenStream::from(tokens)
}
