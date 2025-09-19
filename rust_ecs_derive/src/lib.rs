extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

/// Derive macro for automatically implementing Diffable trait
#[proc_macro_derive(Diffable)]
pub fn derive_diffable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let diff_name = syn::Ident::new(&format!("{}Diff", name), name.span());
    
    match &input.data {
        Data::Struct(data_struct) => {
            if let Fields::Named(fields) = &data_struct.fields {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();
                
                let diff_fields = field_names.iter().zip(field_types.iter()).map(|(name, ty)| {
                    quote! {
                        pub #name: Option<<#ty as crate::Diffable>::Diff>
                    }
                });
                
                let diff_computation = field_names.iter().map(|name| {
                    quote! {
                        #name: {
                            let field_diff = self.#name.diff(&other.#name);
                            if field_diff.is_some() {
                                has_changes = true;
                            }
                            field_diff
                        }
                    }
                });
                
                let apply_diff_operations = field_names.iter().map(|name| {
                    quote! {
                        if let Some(ref field_diff) = diff.#name {
                            self.#name.apply_diff(field_diff);
                        }
                    }
                });
                
                let expanded = quote! {
                    #[derive(Clone, Debug)]
                    pub struct #diff_name {
                        #(#diff_fields,)*
                    }
                    
                    impl crate::Diffable for #name {
                        type Diff = #diff_name;
                        
                        fn diff(&self, other: &Self) -> Option<Self::Diff> {
                            let mut has_changes = false;
                            let diff = Self::Diff {
                                #(#diff_computation,)*
                            };
                            
                            if has_changes {
                                Some(diff)
                            } else {
                                None
                            }
                        }
                        
                        fn apply_diff(&mut self, diff: &Self::Diff) {
                            #(#apply_diff_operations)*
                        }
                    }
                    
                    impl crate::DiffableComponent for #name {}
                };
                
                TokenStream::from(expanded)
            } else {
                panic!("Diffable can only be derived for structs with named fields");
            }
        }
        _ => panic!("Diffable can only be derived for structs"),
    }
}