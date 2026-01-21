use proc_macro2::TokenStream;
use quote::quote;

use crate::user_type::{Attributes, UserType};

pub(crate) fn types(
    field_ident: &syn::Ident,
    field_type: UserType,
    attrs: Attributes,
) -> syn::Result<TokenStream> {
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
}

pub(crate) fn builder(
    field_ident: &syn::Ident,
    field_type: UserType,
    attrs: Attributes,
) -> syn::Result<TokenStream> {
    let is_option = field_type.is_option()?;
    if is_option || attrs.each.is_some() {
        Ok(quote!(
            #field_ident: self.#field_ident.clone()
        ))
    } else {
        Ok(quote! {
            #field_ident: self.#field_ident.clone().ok_or::<::std::string::String>
                ("Field: ".to_owned() + stringify!(#field_ident))?
        })
    }
}

pub(crate) fn inits(
    field_ident: &syn::Ident,
    _: UserType,
    attrs: Attributes,
) -> syn::Result<TokenStream> {
    Ok(if attrs.each.is_some() {
        quote! {
            #field_ident: ::std::vec::Vec::new()
        }
    } else {
        quote! {
            #field_ident: ::std::option::Option::None
        }
    })
}

pub(crate) fn big_setters(
    field_ident: &syn::Ident,
    field_type: UserType,
    attrs: Attributes,
) -> syn::Result<TokenStream> {
    let inner_type = field_type.expected_type()?;
    Ok(if attrs.each.is_some() {
        quote! {
            pub fn #field_ident(&mut self, #field_ident: #inner_type) -> &mut Self {
                self.#field_ident = #field_ident;
                self
            }
        }
    } else {
        quote! {
            pub fn #field_ident(&mut self, #field_ident: #inner_type) -> &mut Self {
                let _ = self.#field_ident.insert(#field_ident);
                self
            }

        }
    })
}

pub(crate) fn small_setters(
    field_ident: &syn::Ident,
    field_type: UserType,
    attrs: Attributes,
) -> syn::Result<TokenStream> {
    let Some(ident) = &attrs.each else {
        return Ok(quote! {});
    };

    if ident == field_ident {
        return Ok(quote! {});
    }

    let inner_opt = field_type.unwrap_vec()?;
    let inner_type: &syn::Type = inner_opt
        .ok_or_else(|| syn::Error::new_spanned(<&syn::Type>::from(field_type), "Should be vec"))?
        .into();

    Ok(quote! {
        pub fn #ident (&mut self, #ident: #inner_type) -> &mut Self {
            self.#field_ident.push(#ident);
            self
        }
    })
}
