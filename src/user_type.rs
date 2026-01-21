use std::marker::PhantomData;

use syn::NestedMeta;

/// Represents the type in a user struct. It maybe wrapped in an Option,
/// so this is my way to guarantee that it is always unwrapped (one way or another).
#[derive(Clone, Copy)]
pub(crate) struct UserType<'a>(&'a syn::Type);
// Should I be borrowing, or act like owning, with #[repr(transparent)]?

impl<'a> From<&'a syn::Type> for UserType<'a> {
    fn from(value: &'a syn::Type) -> Self {
        UserType(value)
    }
}

impl<'a> From<UserType<'a>> for &'a syn::Type {
    fn from(value: UserType<'a>) -> Self {
        value.0
    }
}

impl<'a> UserType<'a> {
    fn unwrap_generic(&self, type_name: &str) -> Option<Self> {
        let syn::Type::Path(type_path) = self.0 else {
            return None;
        };
        let path = &type_path.path;
        let mut segments = path.segments.iter();
        let option = &segments.find(|path_segment| path_segment.ident == type_name)?;

        let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args, ..
        }) = &option.arguments
        else {
            panic!("Option without angle brackets!");
        };
        let Some(syn::GenericArgument::Type(output_type)) = args.first() else {
            panic!("Option without generic argument");
        };
        Some(output_type.into())
    }

    /// Accesses the inner type (if it was actually an option)
    pub(crate) fn unwrap_option(&self) -> Option<Self> {
        self.unwrap_generic("Option")
    }

    pub(crate) fn unwrap_vec(&self) -> Option<Self> {
        self.unwrap_generic("Vec")
    }

    /// Accesses the type, whether it is Option wrapped or not.
    pub(crate) fn expected_type(&self) -> &syn::Type {
        if let Some(inner_type) = self.unwrap_option() {
            inner_type.into()
        } else {
            self.0
        }
    }

    pub(crate) fn is_option(&self) -> bool {
        self.unwrap_option().is_some()
    }

    pub(crate) fn is_vec(&self) -> bool {
        self.unwrap_vec().is_some()
    }
}

/// Ensures types of user fields are always accessed through `UserType`
#[derive(Clone, Copy)]
pub(crate) struct UserFields<'a>(&'a syn::Fields);

impl<'a> From<&'a syn::Fields> for UserFields<'a> {
    fn from(value: &'a syn::Fields) -> Self {
        UserFields(value)
    }
}

impl<'a> UserFields<'a> {
    pub(crate) fn iter(&'a self) -> impl Iterator<Item = (&'a syn::Ident, UserType<'a>)> {
        self.0.iter().map(|field| {
            let field_name = field.ident.as_ref().expect("No name");
            let field_type = &field.ty;

            (field_name, field_type.into())
        })
    }

    pub(crate) fn iter_attrs(
        &'a self,
    ) -> impl Iterator<Item = (&'a syn::Ident, UserType<'a>, Attributes<'a>)> {
        self.0.iter().map(|field| {
            let field_name = field.ident.as_ref().expect("No name");
            let field_type = &field.ty;

            (field_name, field_type.into(), field.attrs.iter().into())
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Attributes<'a> {
    pub(crate) each: Option<syn::Ident>,
    phantom: PhantomData<&'a u32>,
}

impl<'a> Attributes<'a> {
    fn new_each(ident: syn::Ident) -> Self {
        Attributes {
            each: Some(ident),
            phantom: PhantomData {},
        }
    }

    fn parse_each(attr: &'a syn::Attribute) -> syn::Ident {
        let syn::Meta::List(syn::MetaList { path, nested, .. }) =
            attr.parse_meta().expect("Wrong attribute")
        else {
            panic!("Not a meta-list");
        };

        if path.get_ident().expect("hm") != "builder" {
            panic!("Not a builder attribute");
        }

        let Some(nested_meta) = nested.first() else {
            panic!("No name");
        };

        let NestedMeta::Meta(parsed) = nested_meta else {
            panic!("Unexpected literal");
        };

        let syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. }) = parsed else {
            panic!("Not name value");
        };

        if path.get_ident().expect("Not ident") != "each" {
            panic!("unknown attribute");
        }

        let syn::Lit::Str(lit_str) = lit else {
            panic!("not a string literal");
        };

        let token_stream = lit_str
            .value()
            .parse::<proc_macro2::TokenStream>()
            .expect("Tokening failed");

        syn::parse2(token_stream).expect("Parsing failed")
    }
}

impl<'a, AttrsT: Iterator<Item = &'a syn::Attribute>> From<AttrsT> for Attributes<'a> {
    fn from(attrs: AttrsT) -> Self {
        let mut each_iter = attrs.map(Self::parse_each);
        let Some(first_each) = each_iter.next() else {
            return Attributes::default();
        };
        if each_iter.next().is_some() {
            panic!("Multiple each for one member");
        }

        Attributes::new_each(first_each)
    }
}
