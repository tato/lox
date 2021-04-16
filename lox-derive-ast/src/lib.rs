use proc_macro::{TokenStream};
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Ident, Type, Token};
use quote::{quote, format_ident};

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
    
    let structs: proc_macro2::TokenStream = types.iter().map(|ty| {
        let fields: proc_macro2::TokenStream = ty.fields.iter().map(|(ident, ty)| {
            quote! {
                #ident: #ty,
            }
        }).collect();
        let field_names: proc_macro2::TokenStream = ty.fields.iter().map(|(ident, _)| {
            quote! {
                #ident,
            }
        }).collect();
        let ty_name = &ty.name;
        let fn_name = format_ident!("visit_{}", ty.name);
        quote! {
            pub struct #ty_name {
                #fields
            }
            impl #ty_name {
                fn new(#fields) -> Self {
                    Self{ #field_names }
                }
            }
            impl Expr for #ty_name {
                fn accept<R>(&mut self, visitor: &mut Visitor<R>) -> R {
                    visitor.#fn_name(self)
                }
            }
        }
    }).collect();

    let visitor_fns: proc_macro2::TokenStream = types.iter().map(|ty| {
        let ty_name = &ty.name;
        let fn_name = format_ident!("visit_{}", ty.name);
        quote! {
            fn #fn_name(expr: &mut #ty_name) -> R;
        }
    }).collect();

    let expanded = quote! {
        trait Visitor<R> {
            #visitor_fns
        }
        trait Expr {
            fn accept(&mut self, visitor: &mut Visitor<R>) -> R;
        }
        #structs
    };

    expanded.into()
}
