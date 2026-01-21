use syn::DeriveInput;

use crate::builders::impl_derive_builder;

mod builders;
pub(crate) mod quote_user;
pub(crate) mod user_type;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("Not a derive input");

    let name = &ast.ident;
    let syn::Data::Struct(struct_data) = &ast.data else {
        return syn::Error::new_spanned(ast, "Builder works only on types")
            .to_compile_error()
            .into();
    };
    match impl_derive_builder(name, (&struct_data.fields).into()) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
