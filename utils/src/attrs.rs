use crate::*;

#[derive(Clone)]
struct Entry {
    span: Span,
    value: String,
}

#[derive(Clone)]
pub struct Attrs<'a> {
    span: Span,
    name: &'a str,
    keys: &'a [&'a str],
    m: HashMap<&'a str, Entry>,
}

impl<'a> Attrs<'a> {
    pub fn new_with(
        span: Span,
        attrs: &[syn::Attribute],
        name: &'a str,
        keys: &'a [&'a str],
    ) -> Result<Self> {
        Self {
            span,
            name,
            keys,
            m: HashMap::new(),
        }
        .to_new(attrs)
    }

    pub fn to_merged(&self, attrs: &[syn::Attribute]) -> Result<Self> {
        let mut ret = self.clone();
        ret.m.extend(self.to_new(attrs)?.m);
        Ok(ret)
    }

    pub fn to_new(&self, attrs: &[syn::Attribute]) -> Result<Self> {
        self.to_new_with(self.span, attrs, self.keys)
    }

    pub fn to_new_with(
        &self,
        span: Span,
        attrs: &[syn::Attribute],
        keys: &'a [&'a str],
    ) -> Result<Self> {
        let mut ret = Self {
            span,
            name: self.name,
            keys,
            m: HashMap::new(),
        };
        ret.add(attrs)?;
        Ok(ret)
    }

    fn add(&mut self, attrs: &[syn::Attribute]) -> Result<()> {
        for attr in attrs {
            if attr.path.is_ident(self.name) {
                let entries: Punctuated<_, Token!(,)> =
                    attr.parse_args_with(|p: ParseStream| p.parse_terminated(Parse::parse))?;
                for entry in entries {
                    match entry {
                        NestedMeta::Meta(Meta::Path(p)) => {
                            return Err(error!(p, "path not supported"))
                        }
                        NestedMeta::Meta(Meta::List(l)) => {
                            return Err(error!(l, "list not supported"))
                        }
                        NestedMeta::Meta(Meta::NameValue(ref v)) => {
                            if let Some(key) =
                                self.keys.iter().copied().find(|k| v.path.is_ident(k))
                            {
                                let value: syn::LitStr = syn::parse2(v.lit.to_token_stream())?;
                                let new_entry = Entry {
                                    span: value.span(),
                                    value: value.value(),
                                };
                                if let Some(existing) = self.m.get(key) {
                                    let mut a = syn::Error::new(
                                        new_entry.span,
                                        format_args!("Duplicate attribute for {key} 1"),
                                    );
                                    let b = syn::Error::new(
                                        existing.span,
                                        format_args!("Duplicate attribute for {key} 2"),
                                    );
                                    a.combine(b);
                                    return Err(a);
                                }
                                self.m.insert(key, new_entry);
                            } else {
                                return Err(error!(v.path, "unknown attribute argument"));
                            }
                        }
                        NestedMeta::Lit(l) => return Err(error!(l, "lit not supported")),
                    }
                }
            }
        }
        Ok(())
    }

    pub fn opt<'b>(&'b self, name: &str) -> Option<&'b str> {
        self.m.get(name).map(|v| v.value.as_str())
    }

    pub fn span(&self, name: &str) -> Span {
        self.m.get(name).map(|v| v.span).unwrap_or(self.span)
    }

    pub fn get<'b>(&'b self, name: &str, default: &'b str) -> &'b str {
        self.opt(name).unwrap_or(default)
    }
}
