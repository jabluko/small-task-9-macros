use syn::DeriveInput;

use crate::builders::impl_derive_builder;

mod builders;
pub(crate) mod user_type;

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("Not a derive input");

    let name = &ast.ident;
    let syn::Data::Struct(struct_data) = &ast.data else {
        panic!("Not a struct");
    };
    let gen = impl_derive_builder(name, (&struct_data.fields).into());
    gen.into()
}
