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
    fn unwrap_generic(&self, type_name: &str) -> syn::Result<Option<Self>> {
        let syn::Type::Path(type_path) = self.0 else {
            return Ok(None);
        };
        let path = &type_path.path;
        let mut segments = path.segments.iter();
        let option = match segments.find(|path_segment| path_segment.ident == type_name) {
            Some(seg) => seg,
            None => return Ok(None),
        };

        let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args, ..
        }) = &option.arguments
        else {
            return Err(syn::Error::new_spanned(
                self.0,
                format!("{}! without angle brackets", type_name),
            ));
        };
        let Some(syn::GenericArgument::Type(output_type)) = args.first() else {
            return Err(syn::Error::new_spanned(
                self.0,
                format!("{}! without generic argument", type_name),
            ));
        };
        Ok(Some(output_type.into()))
    }

    /// Accesses the inner type (if it was actually an option)
    pub(crate) fn unwrap_option(&self) -> syn::Result<Option<Self>> {
        self.unwrap_generic("Option")
    }

    pub(crate) fn unwrap_vec(&self) -> syn::Result<Option<Self>> {
        self.unwrap_generic("Vec")
    }

    /// Accesses the type, whether it is Option wrapped or not.
    pub(crate) fn expected_type(&self) -> syn::Result<&syn::Type> {
        if let Some(inner_type) = self.unwrap_option()? {
            Ok(inner_type.into())
        } else {
            Ok(self.0)
        }
    }

    pub(crate) fn is_option(&self) -> syn::Result<bool> {
        Ok(self.unwrap_option()?.is_some())
    }

    pub(crate) fn is_vec(&self) -> syn::Result<bool> {
        Ok(self.unwrap_vec()?.is_some())
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
    ) -> impl Iterator<Item = syn::Result<(&'a syn::Ident, UserType<'a>, Attributes<'a>)>> {
        self.0.iter().map(|field| {
            let field_name = field.ident.as_ref().expect("No name");
            let field_type = &field.ty;
            let attrs = Attributes::from_attrs(field.attrs.iter())?;

            Ok((field_name, field_type.into(), attrs))
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

    fn parse_each(attr: &'a syn::Attribute) -> syn::Result<syn::Ident> {
        let syn::Meta::List(syn::MetaList { path, nested, .. }) =
            attr.parse_meta().expect("Wrong attribute")
        else {
            return Err(syn::Error::new_spanned(attr, "Not a meta-list"));
        };

        if path.get_ident().expect("hm") != "builder" {
            return Err(syn::Error::new_spanned(path, "Not a builder attribute"));
        }

        let Some(nested_meta) = nested.first() else {
            return Err(syn::Error::new_spanned(nested, "No name"));
        };

        let NestedMeta::Meta(parsed) = nested_meta else {
            return Err(syn::Error::new_spanned(nested_meta, "Unexpected literal"));
        };

        let syn::Meta::NameValue(syn::MetaNameValue { path, lit, .. }) = parsed else {
            return Err(syn::Error::new_spanned(parsed, "Expected name-value pair"));
        };

        if path.get_ident().expect("Not ident") != "each" {
            return Err(syn::Error::new_spanned(path, "unknown attribute, did you mean: `each`?"));
        }

        let syn::Lit::Str(lit_str) = lit else {
            return Err(syn::Error::new_spanned(lit, "Expected string literal"));
        };

        let token_stream = lit_str
            .value()
            .parse::<proc_macro2::TokenStream>()
            .map_err(|e| syn::Error::new_spanned(lit, e))?;

        Ok(syn::parse2(token_stream)?)
    }
}

impl<'a> Attributes<'a> {
    pub(crate) fn from_attrs(attrs: impl Iterator<Item = &'a syn::Attribute>) -> syn::Result<Self> {
        let mut each_results = attrs.map(|attr| (Self::parse_each(attr), attr));

        let first_each = match each_results.next() {
            None => return Ok(Attributes::default()),
            Some((Err(e), _)) => return Err(e),
            Some((Ok(ident), _)) => ident,
        };

        if let Some((_, attr)) = each_results.next() {
            return Err(syn::Error::new_spanned(
                attr,
                "unexpected second 'each' attribute",
            ));
        }

        Ok(Attributes::new_each(first_each))
    }
}
