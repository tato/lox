use proc_macro::{TokenStream, TokenTree};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Ident, Type, Token};
use quote::quote;

struct AstType {
    name: Ident,
    fields: Vec<(Ident, Type)>,
}
struct AstTypes {
    types: Vec<AstType>
}
impl Parse for AstTypes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut types = Vec::new();
        loop {
            if input.is_empty() {
                break
            }

            let name: Ident = input.parse()?;
            input.parse::<Token![=>]>()?;
            let mut fields = Vec::new();
            loop {
                let field: Ident = input.parse()?;
                input.parse::<Token![:]>()?;
                let ty: Type = input.parse()?;
                fields.push((field, ty));

                let lookahead = input.lookahead1();
                if lookahead.peek(Token![;]) {
                    input.parse::<Token![;]>()?;
                    break
                } else if lookahead.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
            }
            types.push(AstType{ name, fields })
        }
        Ok(AstTypes{ types })
    }
}

#[proc_macro]
pub fn make_ast(input: TokenStream) -> TokenStream {
    let AstTypes { types } = parse_macro_input!(input as AstTypes);
    
    let expanded = types.iter().map(|at| {
        XD
        TokenStream::from(quote! {
            struct #at.name {

            }
        })
    });

    expanded.collect()
}
