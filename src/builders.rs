use quote::quote;
use syn::{Ident, Type};

use crate::user_type::UserFields;

pub(crate) fn impl_derive_builder(
    name: &Ident,
    user_fields: UserFields,
) -> proc_macro2::TokenStream {
    let builder_struct = struct_decl(user_fields);
    let builder_impl = builder_impl(name, user_fields);

    let field_inits = user_fields.iter_attrs().map(|(field_ident, _, attrs)| {
        if attrs.each.is_some() {
            quote! {
                #field_ident: Vec::new()
            }
        } else {
            quote! {
                #field_ident: None
            }
        }
    });

    let user_impl = quote! {
        impl #name {
            pub fn builder() -> Builder {
                Builder {
                    #(#field_inits,)*
                }
            }
        }
    };

    //eprintln!("TOKENS: {}", user_impl);

    let all = quote! {
        #builder_struct
        #builder_impl
        #user_impl
    };

    all
}

fn struct_decl(user_fields: UserFields) -> proc_macro2::TokenStream {
    let fields = user_fields
        .iter_attrs()
        .map(|(field_ident, field_type, attrs)| {
            let inner_type = field_type.expected_type();
            if attrs.each.is_some() {
                quote! {
                    #field_ident: #inner_type
                }
            } else {
                quote! {
                    #field_ident: Option<#inner_type>
                }
            }
        });

    quote! {
        struct Builder {
            #(#fields,)*
        }
    }
}

fn builder_impl(name: &Ident, user_fields: UserFields) -> proc_macro2::TokenStream {
    let setters = user_fields
        .iter_attrs()
        .map(|(field_name, field_type, attr)| {
            let inner_type = field_type.expected_type();
            if attr.each.is_some() {
                quote! {
                    pub fn #field_name(&mut self, #field_name: #inner_type) -> &mut Self {
                        self.#field_name = #field_name;
                        self
                    }
                }
            } else {
                quote! {
                    pub fn #field_name(&mut self, #field_name: #inner_type) -> &mut Self {
                        let _ = self.#field_name.insert(#field_name);
                        self
                    }
                }
            }
        });

    let single_setters = user_fields
        .iter_attrs()
        .map(|(field_name, field_type, attrs)| {
            let Some(ident) = &attrs.each else {
                return quote!{};
            };

            if ident == field_name {
                return quote!{};
            }

            let inner_type: &Type = field_type.unwrap_vec().expect("Should be vec").into();

            quote! {
                pub fn #ident (&mut self, #ident: #inner_type) -> &mut Self {
                    self.#field_name.push(#ident);
                    self
                }
            }
        });

    let inits = user_fields
        .iter_attrs()
        .map(|(field_name, field_type, attrs)| {
            if field_type.is_option() || attrs.each.is_some() {
                quote!(
                    #field_name: self.#field_name.clone()
                )
            } else {
                quote! {
                    #field_name: self.#field_name.clone().ok_or::<String>
                        ("Field: ".to_owned() + stringify!(#field_name))?
                }
            }
        });

    let build_fn = quote! {
        fn build(&mut self) -> Result<#name, Box<dyn ::std::error::Error>> {
            Ok(#name {
                #(#inits,)*
            })
        }
    };

    let output = quote! {
        impl Builder {
            #(#setters)*
            #(#single_setters)*
            #build_fn
        }
    };

    eprintln!("TOKENS: {}", output);
    output
}
