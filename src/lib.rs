use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    BoundLifetimes, ItemFn, Token, TraitBound, Type, WherePredicate,
};

#[proc_macro_attribute]
/// Allows users to use repeated trait bound for any number of types
///
/// Rather than having to write
/// ```rust
/// fn my_func<T>() -> T
/// where
///     String: SomeTrait<T>,
///     for <'a> &'a str: SomeTrait<T>
/// {
///     ...
/// }
/// ```
///
/// You can instead write
/// ```rust
/// #[bounded(String, for<'a> &'a str: SomeTrait<T>)]
/// fn my_func<T>() -> T {
///     ...
/// }
/// ```
pub fn each(attr: TokenStream, item: TokenStream) -> TokenStream {
    let bounds = parse_macro_input!(attr as RepeatedTraitBounds);
    let item = parse_macro_input!(item as TokenStream2);

    match each_impl(bounds, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

fn each_impl(bounds: RepeatedTraitBounds, item: TokenStream2) -> syn::Result<TokenStream2> {
    let mut func = syn::parse2::<ItemFn>(item)?;

    bounds.into_trait_bounds().try_for_each(|token_stream| {
        match syn::parse2::<WherePredicate>(token_stream) {
            Ok(p) => {
                func.sig.generics.make_where_clause().predicates.push(p);

                Ok(())
            }
            Err(e) => Err(syn::Error::new(e.span(), e)),
        }
    })?;

    Ok(quote_spanned!(func.span()=> #func))
}

struct BoundType(Option<BoundLifetimes>, Type);

impl Parse for BoundType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: think about how to handle this better, this is an all or nothing approach
        // and doesn't account for malformed syntax, which should return the Err variant
        let bound_lifetime = input.parse::<BoundLifetimes>().ok();
        let bound_type = input.parse::<Type>()?;

        Ok(BoundType(bound_lifetime, bound_type))
    }
}

struct RepeatedTraitBounds {
    types: Vec<BoundType>,
    bounds: Punctuated<TraitBound, Token![+]>,
}

impl RepeatedTraitBounds {
    fn into_trait_bounds(self) -> impl Iterator<Item = TokenStream2> {
        let bounds = self.bounds;

        self.types.into_iter().map(move |BoundType(maybe_lt, ty)| {
            let ty = quote_spanned! {ty.span()=> #ty};

            match maybe_lt {
                Some(lt) => {
                    let lt = quote_spanned! {lt.span()=> #lt};
                    quote!(#lt #ty: #bounds)
                }
                None => quote!(#ty: #bounds),
            }
        })
    }
}

impl Parse for RepeatedTraitBounds {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let types = input
            .call(Punctuated::<BoundType, Token![,]>::parse_separated_nonempty)?
            .into_iter()
            .collect::<Vec<BoundType>>();

        input.parse::<Token![:]>()?;

        let bounds = input.call(Punctuated::<TraitBound, Token![+]>::parse_separated_nonempty)?;

        Ok(RepeatedTraitBounds { types, bounds })
    }
}
