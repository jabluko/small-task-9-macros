use proc_macro2::Ident;
use quote::quote;
use syn::{DataStruct, DeriveInput, Type};

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("Not a derive input");

    let name = &ast.ident;
    let syn::Data::Struct(struct_data) = &ast.data else {
        panic!("Not a struct");
    };
    let gen = impl_derive_builder(name, struct_data);
    gen.into()
}

fn get_fields(struct_data: &DataStruct) -> impl Iterator<Item = (&Ident, &Type)> {
    struct_data.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("No name");
        let field_type = &field.ty;
        (field_name, field_type)
    })
}

fn impl_derive_builder(name: &Ident, struct_data: &DataStruct) -> proc_macro2::TokenStream {
    let builder_struct = struct_decl(struct_data);
    let builder_impl = builder_impl(name, struct_data);

    let field_inits = struct_data.fields.iter().map(|field| {
        let field_name = field.ident.as_ref().expect("No name");
        quote! {
            #field_name: None
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

fn struct_decl(struct_data: &DataStruct) -> proc_macro2::TokenStream {
    let fields = get_fields(struct_data).map(|(field_name, field_type)| {
        quote! {
            #field_name: Option<#field_type>
        }
    });

    quote! {
        struct Builder {
            #(#fields,)*
        }
    }
}

fn builder_impl(name: &Ident, struct_data: &DataStruct) -> proc_macro2::TokenStream {
    let setters = get_fields(struct_data).map(|(field_name, field_type)| {
        quote! {
            pub fn #field_name (&mut self, #field_name: #field_type) {
                let _ = self.#field_name.insert(#field_name);
            }
        }
    });

    let inits = get_fields(struct_data).map(|(field_name, _field_data)| {
        quote! {
            #field_name: self.#field_name.ok_or::<Box<dyn ::std::error::Error>>
                (("Field: ".to_owned() + stringify!(#field_name)).into())?
        }
    });

    let build_fn = quote! {
        fn build(self) -> Result<#name, Box<dyn ::std::error::Error>> {
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
