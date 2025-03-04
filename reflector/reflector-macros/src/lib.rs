extern crate proc_macro;
use core::panic;

use proc_macro::TokenStream;
use syn::{parse::{Parse, ParseStream}, Data, Token};
use quote::{quote, ToTokens};


struct MyParams(syn::Ident, syn::Ident);
impl Parse for MyParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        // let content;
        panic!("CONTENT: {input}");
        // syn::parenthesized!(content in input);

        // let type1 = content.parse()        ;
        // content.parse::<Token![,]>()?;
        // let type2 = content.parse()?;

        // Ok(MyParams(type1, type2))
    }
}


fn derive(ast: &syn::DeriveInput) -> TokenStream {
    let attribute = ast.attrs.iter().filter(
        |a| a.path().segments.len() == 1 && a.path().segments[0].ident == "my_trait"
    ).nth(0).expect("my_trait attribute required for deriving MyTrait!");

    let parameters: MyParams = syn::parse2(attribute.to_token_stream()).expect("Invalid my_trait attribute!");
    quote! {

    }.into()
}



#[proc_macro_derive(MyTrait, attributes(my_trait))]
pub fn system(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let ts = derive(&input);
    let struct_identifier = &input.ident;

    match &input.data {
        Data::Struct(syn::DataStruct { fields, .. }) => {
            quote! {
                ts
                // #[automatically_derived]
                // impl Handler<CombinedMessage> for #struct_identifier {
                //     fn handle(&self, core: & Core, message: & CombinedMessage) {
                //         handle![{self, core, message}
                //             CombinedMessage::Foo
                //             CombinedMessage::Bar
                //             ];                
                //     }    
                // }
            }
        }
        _ => unimplemented!()
    }.into()
}