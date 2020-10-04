extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

fn wrap_type(ty: &syn::Type, wrapping_type: &str) -> proc_macro2::TokenStream {
    let wt_ident = syn::Ident::new(wrapping_type, ty.span());
    quote! { #wt_ident<#ty> }
}

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    // println!("{:#?}", input);

    if let syn::DeriveInput { ref vis, ref ident, data: syn::Data::Struct(syn::DataStruct { ref fields, .. }), .. } = input {
        let b_ident = syn::Ident::new(format!("{}Builder", ident.to_string()).as_str(), ident.span());

        let b_new_fields: Vec<_> = fields.iter().map(|f| {
            match f {
                syn::Field { ident: Some(ref ident), .. } => {
                    let f = quote! {
                        #ident : None
                    };
                    f
                }
                _ => panic!(),
            }
        }).collect();

        let b_build_fields: Vec<_> = fields.iter().map(|f| {
            match f {
                syn::Field { ident: Some(ref ident), .. } => {
                    let f = quote! {
                        #ident : self.#ident.clone().ok_or(format!("Field '{}' was not specified", stringify!(#ident)))?
                    };
                    f
                }
                _ => panic!(),
            }
        }).collect();

        let b_fields: Vec<_> = fields.iter().map(|f| {
            match f {
                syn::Field { ref vis, ident: Some(ref ident), ref ty, .. } => {
                    let ty = wrap_type(ty, "Option");
                    let f = quote! {
                        #vis #ident : #ty
                    };
                    f
                }
                _ => panic!(),
            }
        }).collect();

        let b_methods: Vec<_> = fields.iter().map(|f| {
            match f {
                syn::Field { ref vis, ident: Some(ref ident), ref ty, .. } => {
                    let m = quote! {
                        #vis fn #ident(&mut self, #ident: #ty) -> &mut Self {
                            self.#ident = Some(#ident);
                            self
                        }
                    };
                    m
                }
                _ => panic!(),
            }
        }).collect();

        let expanded = quote! {
            impl #ident {
                #vis fn builder() -> #b_ident {
                    #b_ident {
                        #( #b_new_fields ), *
                    }
                }
            }

            #vis struct #b_ident {
                #( #b_fields), *
            }

            impl #b_ident {
                #( #b_methods) *

                #vis fn build(&self) -> Result<#ident, Box<dyn std::error::Error>> {
                    Ok(#ident {
                        #( #b_build_fields), *
                    })
                }
            }
        };

        expanded.into()
    }
    else { panic!("This macro has to be used on structs") }
}
