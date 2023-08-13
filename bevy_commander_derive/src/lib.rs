// I'm gonna be completely honest, I have no idea how to write a proc macro.
// I just copied this from bevy_console_derive.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(AppCommand, attributes(command))]
pub fn derive_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name_string = get_command_name(&input);
    let name = &input.ident;
    let generics = input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    
    TokenStream::from(quote! {
        impl #impl_generics bevy_commander::AppCommand for #name #type_generics #where_clause {
            fn name() -> &'static str {
                #name_string
            }
        }

        impl #impl_generics bevy::prelude::Resource for #name #type_generics #where_clause {};
    })
}

fn get_command_name(input: &DeriveInput) -> syn::LitStr {
    input
        .attrs
        .iter()
        .find_map(|attr| {
            if attr.path.is_ident("command") {
                if let Ok(syn::Meta::List(list)) = attr.parse_meta() {
                    return list.nested.iter().find_map(|meta| {
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) = meta {
                            Some(nv.lit.clone())
                        } else {
                            None
                        }
                    });
                }
            }
            None
        })
        .map(|lit| {
            if let syn::Lit::Str(str) = lit {
                str
            } else {
                panic!("expected string literal as command name");
            }
        })
        .unwrap_or_else(|| syn::LitStr::new(&input.ident.to_string(), input.ident.span()))
}
