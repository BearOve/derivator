use derivator_utils::*;
use proc_macro::TokenStream;

#[proc_macro_derive(Display, attributes(display))]
pub fn derive_display(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match derive_debug_or_display(input, "Display", "display") {
        Ok(ret) => ret,
        Err(ret) => ret.into_compile_error(),
    }
    .into()
}

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive_debug(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match derive_debug_or_display(input, "Debug", "debug") {
        Ok(ret) => ret,
        Err(ret) => ret.into_compile_error(),
    }
    .into()
}

fn derive_debug_or_display(
    input: DeriveInput,
    trait_name: &str,
    attr_name: &str,
) -> Result<TokenStream2, Error> {
    let trait_name = Ident::new(trait_name, input.ident.span());

    let fmt_impl = match input.data {
        syn::Data::Struct(d) => StructGen {
            ident: &input.ident,
            attrs: &[&input.attrs],
            attr_name,
            prefix: quote!(let Self),
            midfix: quote!(= self;),
        }
        .generate(&d.fields)?,
        syn::Data::Enum(d) => {
            let mut arms = Vec::new();
            for variant in d.variants {
                let ident = &variant.ident;

                arms.push(
                    StructGen {
                        ident,
                        attrs: &[&input.attrs, &variant.attrs],
                        attr_name,
                        prefix: quote!(Self::#ident),
                        midfix: quote!(=>),
                    }
                    .generate(&variant.fields)?,
                );
            }
            quote!(match self { #(#arms,)* })
        }
        syn::Data::Union(d) => return Err(error!(d.union_token.span(), "not supported")),
    };
    let ty_name = input.ident;
    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();
    let ret = quote!(
        impl #impl_gen std::fmt::#trait_name for #ty_name #ty_gen #where_clause {
            #[allow(clippy::uninlined_format_args)]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                #fmt_impl
            }
        }
    );
    if let Ok("yes") = std::env::var("DUMP_DERIVE").as_deref() {
        println!("{}", ret.to_string().replace('\n', ""));
    }
    Ok(ret)
}
