use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(U8Enum)]
pub fn derive_enum_variant_count(input: TokenStream) -> TokenStream {
    let syn_item: syn::DeriveInput = syn::parse(input).unwrap();
    let len = match syn_item.data {
        syn::Data::Enum(enum_item) => enum_item.variants.len(),
        _ => panic!("U8Enum only works on Enums"),
    };
    let enum_name = syn_item.ident;
    let expanded = quote! {
        impl #enum_name {
            pub fn count() -> usize {
                #len
            }
            pub fn as_u8(&self) -> u8 {
                *self as u8
            }
            pub fn from_u8(byte: u8) -> Option<Self> {
                if byte as usize >= Self::count() {
                    None
                } else {
                    unsafe { std::mem::transmute(byte) }
                }
            }
        }
    };
    expanded.into()
}
