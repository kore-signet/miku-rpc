use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    bracketed, parse_macro_input,
    token::{self, Comma},
    Attribute, FnArg, Ident, LitStr, Pat, Path, Result, ReturnType, Signature, Token,
};

mod kw {
    syn::custom_keyword!(docs);
}

const OC2_DOC_BASE: &str =
    "https://github.com/fnuecke/oc2/blob/1.18-forge/src/main/resources/assets/oc2/doc/en_us/";

struct FnDef(Signature, Vec<Attribute>);
impl Parse for FnDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let signature = input.parse::<Signature>()?;
        input.parse::<Token![;]>()?;
        Ok(FnDef(signature, attrs))
    }
}

struct OC2RpcDef {
    oc_method_name: LitStr,
    doc_path: Option<LitStr>,
}

impl Parse for OC2RpcDef {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(OC2RpcDef {
            oc_method_name: input.parse::<LitStr>()?,
            doc_path: if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                input.parse::<kw::docs>()?;
                input.parse::<Token![=]>()?;
                Some(input.parse::<LitStr>()?)
            } else {
                None
            },
        })
    }
}

#[proc_macro_attribute]
pub fn rpc(attr: TokenStream, input: TokenStream) -> TokenStream {
    let OC2RpcDef {
        oc_method_name,
        doc_path,
    } = parse_macro_input!(attr as OC2RpcDef);
    let FnDef(
        Signature {
            ident,
            generics,
            inputs,
            output,
            ..
        },
        attrs,
    ) = parse_macro_input!(input as FnDef);
    let where_clause = &generics.where_clause;

    let ret_type = match output {
        ReturnType::Default => quote! { Option<()> },
        ReturnType::Type(_, t) => quote! { #t },
    };

    let doc_path = doc_path.map(|path| {
        let doc_url = format!("[OC2 Docs]({}/{}):", OC2_DOC_BASE, path.value());
        quote! {
            #[doc = #doc_url]
        }
    });

    let arg_idents: Vec<Box<Pat>> = inputs
        .iter()
        .filter_map(|v| {
            if let FnArg::Typed(t) = v {
                Some(t.pat.clone())
            } else {
                None
            }
        })
        .collect();
    let arg_defs = inputs.into_iter();

    let tokens = quote! {
        #doc_path
        #(#attrs)*
        fn #ident #generics (&self, bus: &mut crate::DeviceBus, #(#arg_defs),*) -> std::io::Result<#ret_type> #where_clause {
            let response: crate::Response<#ret_type> = bus.call(&crate::Call::invoke(self.id(), #oc_method_name, &[#(&#arg_idents),*]))?;
            Ok(response.data)
        }
    };

    TokenStream::from(tokens)
}

struct DeviceData {
    rust_name: Ident,
    oc2_identity: LitStr,
    docs: LitStr,
    _bracket_token: token::Bracket,
    capabilities: Punctuated<Path, Comma>,
}

impl Parse for DeviceData {
    fn parse(input: ParseStream) -> Result<Self> {
        let rust_name = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let oc2_identity = input.parse::<LitStr>()?;
        input.parse::<Comma>()?;
        let docs = input.parse::<LitStr>()?;
        input.parse::<Comma>()?;

        let content;
        Ok(DeviceData {
            rust_name,
            oc2_identity,
            docs,
            _bracket_token: bracketed!(content in input),
            capabilities: content.parse_terminated(Path::parse)?,
        })
    }
}

#[proc_macro]
pub fn define_device(tokens: TokenStream) -> TokenStream {
    let DeviceData {
        rust_name,
        docs,
        oc2_identity,
        capabilities,
        ..
    } = parse_macro_input!(tokens as DeviceData);

    let capabilities = capabilities.into_iter();

    let tokens = quote! {
        #[doc = #docs ]
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
