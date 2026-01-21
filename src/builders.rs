use quote::quote;
use syn::{Ident, Type};

use crate::user_type::UserFields;

pub(crate) fn impl_derive_builder(
    name: &Ident,
    user_fields: UserFields,
) -> syn::Result<proc_macro2::TokenStream> {
    let builder_struct = struct_decl(user_fields)?;
    let builder_impl = builder_impl(name, user_fields)?;

    let field_inits: Vec<_> = user_fields
        .iter_attrs()
        .map(|result| {
            result.map(|(field_ident, _, attrs)| {
                if attrs.each.is_some() {
                    quote! {
                        #field_ident: ::std::vec::Vec::new()
                    }
                } else {
                    quote! {
                        #field_ident: ::std::option::Option::None
                    }
                }
            })
        })
        .collect::<syn::Result<_>>()?;

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

    Ok(all)
}

fn struct_decl(user_fields: UserFields) -> syn::Result<proc_macro2::TokenStream> {
    let fields: Vec<_> = user_fields
        .iter_attrs()
        .map(|result| {
            result.and_then(|(field_ident, field_type, attrs)| {
                let inner_type = field_type.expected_type()?;
                Ok(if attrs.each.is_some() {
                    quote! {
                        #field_ident: #inner_type
                    }
                } else {
                    quote! {
                        #field_ident: ::std::option::Option<#inner_type>
                    }
                })
            })
        })
        .collect::<syn::Result<_>>()?;

    Ok(quote! {
        struct Builder {
            #(#fields,)*
        }
    })
}

fn builder_impl(name: &Ident, user_fields: UserFields) -> syn::Result<proc_macro2::TokenStream> {
    let setters: Vec<_> = user_fields
        .iter_attrs()
        .map(|result| {
            result.and_then(|(field_name, field_type, attr)| {
                let inner_type = field_type.expected_type()?;
                Ok(if attr.each.is_some() {
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
                })
            })
        })
        .collect::<syn::Result<_>>()?;

    let single_setters: Vec<_> = user_fields
        .iter_attrs()
        .map(|result| {
            result.and_then(|(field_name, field_type, attrs)| {
                let Some(ident) = &attrs.each else {
                    return Ok(quote!{});
                };

                if ident == field_name {
                    return Ok(quote!{});
                }

                let inner_opt = field_type.unwrap_vec()?;
                let inner_type: &Type = inner_opt.ok_or_else(|| {
                    syn::Error::new_spanned(
                        <&syn::Type>::from(field_type),
                        "Should be vec"
                    )
                })?.into();

                Ok(quote! {
                    pub fn #ident (&mut self, #ident: #inner_type) -> &mut Self {
                        self.#field_name.push(#ident);
                        self
                    }
                })
            })
        })
        .collect::<syn::Result<_>>()?;

    let inits: Vec<_> = user_fields
        .iter_attrs()
        .map(|result| {
            result.and_then(|(field_name, field_type, attrs)| {
                let is_option = field_type.is_option()?;
                if is_option || attrs.each.is_some() {
                    Ok(quote!(
                        #field_name: self.#field_name.clone()
                    ))
                } else {
                    Ok(quote! {
                        #field_name: self.#field_name.clone().ok_or::<::std::string::String>
                            ("Field: ".to_owned() + stringify!(#field_name))?
                    })
                }
            })
        })
        .collect::<syn::Result<_>>()?;

    let build_fn = quote! {
        fn build(&mut self) -> ::std::result::Result<#name, ::std::boxed::Box<dyn ::std::error::Error>> {
            ::std::result::Result::Ok(#name {
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
    Ok(output)
}
