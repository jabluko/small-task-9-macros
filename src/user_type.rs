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

impl<'a> UserType<'a> {
    /// Accesses the inner type (if it was actually an option)
    pub(crate) fn unwrap_option(&self) -> Option<&syn::Type> {
        let syn::Type::Path(type_path) = self.0 else {
            return None;
        };
        let path = &type_path.path;
        let mut segments = path.segments.iter();
        let option = &segments
            .find(|path_segment| {
                path_segment.ident == "Option"
            })?;

        let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            args, ..
        }) = &option.arguments
        else {
            panic!("Option without angle brackets!");
        };
        let Some(syn::GenericArgument::Type(output_type)) = args.first() else {
            panic!("Option without generic argument");
        };
        Some(output_type)
    }

    /// Accesses the type, whether it is Option wrapped or not.
    pub(crate) fn unwrap(&self) -> &syn::Type {
        if let Some(inner_type) = self.unwrap_option() {
            inner_type
        } else {
            self.0
        }
    }

    pub(crate) fn is_option(&self) -> bool {
        self.unwrap_option().is_some()
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
}
