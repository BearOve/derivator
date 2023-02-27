use crate::*;

macro_rules! format_arg {
    ($attrs:expr, $($arg:tt)+) => {{
        if let Some(format) = $attrs.opt("format") {
            quote!(format_args!(#format, $($arg)+))
        } else {
            quote!($($arg)+)
        }
    }};
}

pub struct StructGen<'a> {
    pub ident: &'a Ident,
    pub attrs: &'a [&'a [syn::Attribute]],
    pub attr_name: &'a str,
    pub prefix: TokenStream2,
    pub midfix: TokenStream2,
}

impl<'a> StructGen<'a> {
    pub fn generate(&self, fields: &syn::Fields) -> Result<TokenStream2> {
        match fields {
            syn::Fields::Named(fields) => self.generate_named(fields),
            syn::Fields::Unnamed(fields) => self.generate_unnamed(fields),
            syn::Fields::Unit => self.generate_unit(),
        }
    }

    fn make_attrs(&self, keys: &'static [&'static str]) -> Result<Attrs<'a>> {
        let mut a = Attrs::new_with(self.ident.span(), self.attrs[0], self.attr_name, keys)?;
        for attrs in &self.attrs[1..] {
            a = a.to_merged(attrs)?;
        }
        Ok(a)
    }

    fn generate_named(&self, fields: &syn::FieldsNamed) -> Result<TokenStream2> {
        let Self {
            ident: _,
            attrs: _,
            attr_name: _,
            prefix,
            midfix,
        } = self;
        let attrs = self.make_attrs(&["format", "ignore", "mode"])?;
        let ignore: HashSet<_> = attrs
            .opt("ignore")
            .map(|v| v.split(',').map(str::trim))
            .into_iter()
            .flatten()
            .collect();

        match attrs.opt("mode") {
            Some("self") => {
                let format = attrs.get("format", "{self}");
                return Ok(quote!(write!(f, #format, self=self)));
            }
            Some(unk) => return Err(error!(attrs.span("mode"), "Unknown mode {unk:?}")),
            None => {}
        }

        let mut deconstruct_fields = Vec::new();
        let mut arg_names = Vec::new();
        let mut field_values = Vec::new();
        for field in &fields.named {
            let name = field.ident.as_ref().unwrap();
            let field_attrs =
                attrs.to_new_with(field.ident.span(), &field.attrs, &["format", "mode"])?;
            if ignore.contains(name.to_string().as_str()) {
                deconstruct_fields.push(quote!(#name: _));
                continue;
            } else {
                deconstruct_fields.push(quote!(#name));
                arg_names.push(name);
            }

            let format_item = if let Some(field_format) = field_attrs.opt("format") {
                quote!(format_args!(#field_format, #name))
            } else {
                quote!(#name)
            };

            match field_attrs.opt("mode") {
                None => field_values.push(format_item),
                Some("iter_concat") => {
                    field_values.push(quote!(DisplayConcat(#name, |#name, f: &mut std::fmt::Formatter| write!(f, "{}", #format_item))));
                }
                unk => return Err(error!(field_attrs.span("mode"), "Unknown mode {unk:?}")),
            }
        }

        if let Some(format) = attrs.opt("format") {
            Ok(quote!(
                #prefix { #(#deconstruct_fields,)* } #midfix {
                    write!(f, #format, #(#arg_names=#field_values,)*)
                }
            ))
        } else if fields.named.len() == 1 {
            Ok(quote!(
                let Self { #(#deconstruct_fields,)* } = self;
                write!(f, "{0}", #(#field_values,)*)
            ))
        } else {
            Err(error!(
                self.ident,
                "Missing #[format(...)] for struct with multiple fields"
            ))
        }
    }

    fn generate_unnamed(&self, fields: &syn::FieldsUnnamed) -> Result<TokenStream2> {
        let Self {
            ident: _,
            attrs: _,
            attr_name,
            prefix,
            midfix,
        } = self;
        let attrs = self.make_attrs(&["format", "ignore"])?;
        let ignore: HashSet<_> = attrs
            .opt("ignore")
            .map(|v| v.split(',').map(str::trim).map(|v| v.parse::<usize>()))
            .into_iter()
            .flatten()
            .collect::<Result<_, _>>()
            .map_err(|e| -> Error {
                error!(
                    attrs.span("ignore"),
                    "Failed to parse index in attribute ignore: {e}"
                )
            })?;
        let mut field_names = Vec::new();
        let mut format_args = Vec::new();
        for (i, field) in fields.unnamed.iter().enumerate() {
            let field_attrs = Attrs::new_with(field.span(), &field.attrs, attr_name, &["format"])?;
            let field_name = syn::Ident::new(&format!("f{i}"), field.span());
            if ignore.contains(&i) {
                field_names.push(quote!(_));
            } else {
                field_names.push(quote!(#field_name));
                format_args.push(format_arg!(field_attrs, #field_name));
            }
        }

        let format = attrs.get("format", "{}");
        let arg = quote!(write!(f, #format, #(#format_args),*));

        Ok(quote!(#prefix (#(#field_names,)*) #midfix #arg))
    }

    fn generate_unit(&self) -> Result<TokenStream2> {
        let Self {
            ident,
            attrs: _,
            attr_name: _,
            prefix,
            midfix,
        } = self;
        let attrs = self.make_attrs(&["format"])?;
        let name = ident.to_string().to_lowercase();

        let format = attrs.get("format", "{}");
        let arg = quote!(write!(f, #format, #name));

        Ok(quote!(#prefix #midfix #arg))
    }
}
