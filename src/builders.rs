use quote::quote;
use syn::Ident;

use crate::{quote_user, user_type::UserFields};

pub(crate) fn impl_derive_builder(
    name: &Ident,
    user_fields: UserFields,
) -> syn::Result<proc_macro2::TokenStream> {
    let builder_struct = struct_decl(user_fields)?;
    let builder_impl = builder_impl(name, user_fields)?;

    let field_inits = user_fields.iter_attrs(quote_user::inits)?;

    let user_impl = quote! {
        impl #name {
            pub fn builder() -> Builder {
                Builder {
                    #(#field_inits,)*
                }
            }
        }
    };

    Ok(quote! {
        #builder_struct
        #builder_impl
        #user_impl
    })
}

fn struct_decl(user_fields: UserFields) -> syn::Result<proc_macro2::TokenStream> {
    let fields = user_fields.iter_attrs(quote_user::types)?;

    Ok(quote! {
        struct Builder {
            #(#fields,)*
        }
    })
}

fn builder_impl(name: &Ident, user_fields: UserFields) -> syn::Result<proc_macro2::TokenStream> {
    let setters = user_fields.iter_attrs(quote_user::big_setters)?;
    let single_setters = user_fields.iter_attrs(quote_user::small_setters)?;
    let inits = user_fields.iter_attrs(quote_user::builder)?;

    let build_fn = quote! {
        fn build(&mut self) -> ::std::result::Result<#name, ::std::boxed::Box<dyn ::std::error::Error>> {
            ::std::result::Result::Ok(#name {
                #(#inits,)*
            })
        }
    };

    Ok(quote! {
        impl Builder {
            #(#setters)*
            #(#single_setters)*
            #build_fn
        }
    })
}
