use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};

struct StaticStmt {
    vis: syn::Visibility,
    #[allow(dead_code)]
    static_token: syn::token::Static,
    ident: syn::Ident,
    #[allow(dead_code)]
    colon_token: syn::token::Colon,
    ty: syn::Type,
    #[allow(dead_code)]
    semi_token: syn::token::Semi,
}

impl Parse for StaticStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(StaticStmt {
            vis: input.parse()?,
            static_token: input.parse()?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
            semi_token: input.parse()?,
        })
    }
}

#[proc_macro_attribute]
pub fn give_away_cell(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let StaticStmt {vis, ident, ty, ..} = syn::parse_macro_input!(input as StaticStmt);

    let output = quote!{
        #[allow(non_snake_case)]
        #vis mod #ident {
            pub(super) struct Token;
            pub(super) static CELL: ::embed_init::GiveAwayCell<Token> = ::embed_init::GiveAwayCell::new();
        }
        impl ::embed_init::CellToken for #ident::Token {
            type Inner = #ty;
        }
    };
    output.into()
}

#[proc_macro_attribute]
pub fn shared_cell(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let StaticStmt {vis, ident, ty, ..} = syn::parse_macro_input!(input as StaticStmt);

    let output = quote!{
        #[allow(non_snake_case)]
        #vis mod #ident {
            pub(crate) struct Token;
            pub(crate) static CELL: ::embed_init::SharedCell<Token> = ::embed_init::SharedCell::new();
        }
        impl ::embed_init::CellToken for #ident::Token {
            type Inner = #ty;
        }
    };
    output.into()
}
