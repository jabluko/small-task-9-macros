use quote::quote;
use syn::Ident;

use crate::user_type::UserFields;

pub(crate) fn impl_derive_builder(
    name: &Ident,
    user_fields: UserFields,
) -> proc_macro2::TokenStream {
    let builder_struct = struct_decl(user_fields);
    let builder_impl = builder_impl(name, user_fields);

    let field_inits = user_fields.iter().map(|(field_ident, _)| {
        quote! {
            #field_ident: None
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
    let fields = user_fields.iter().map(|(field_ident, field_type)| {
        let inner_type = field_type.unwrap();
        quote! {
            #field_ident: Option<#inner_type>
        }
    });

    quote! {
        struct Builder {
            #(#fields,)*
        }
    }
}

fn builder_impl(name: &Ident, user_fields: UserFields) -> proc_macro2::TokenStream {
    let setters = user_fields.iter().map(|(field_name, field_type)| {
        let inner_type = field_type.unwrap();
        quote! {
            pub fn #field_name (&mut self, #field_name: #inner_type) -> &mut Self {
                let _ = self.#field_name.insert(#field_name);
                self
            }
        }
    });

    let inits = user_fields.iter().map(|(field_name, _field_data)| {
        quote! {
            #field_name: self.#field_name.clone().ok_or::<Box<dyn ::std::error::Error>>
                (("Field: ".to_owned() + stringify!(#field_name)).into())?
        }
    });

    let build_fn = quote! {
        fn build(&mut self) -> Result<#name, Box<dyn ::std::error::Error>> {
            Ok(#name {
                #(#inits,)*
            })
        }
    };

    quote! {
        impl Builder {
            #(#setters)*
            #build_fn
        }
    }
}
