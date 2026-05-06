use proc_macro::TokenStream;

/// TODO
/// test using macro twice (expect error)
/// ensure the enum provided is defined as pub?
/// test not using macro
/// test giving a non enum
/// test a payload containing a struct from another module

#[proc_macro_attribute]
pub fn payloads(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemEnum);
    let variant_count: usize = get_variant_count(&input);
    let enum_ident = input.clone().ident;

    // // Ensure the enum provided is defined as pub, else postmaster won't be able to see it
    // if !matches!(input.vis, syn::Visibility::Public(_)) {
    //     return syn::Error::new_spanned(
    //         input.ident,
    //         "The `post_haste::payloads` macro must be used on a enum which is declared with `pub`",
    //     )
    //     .into_compile_error()
    //     .into();
    // }

    let output = quote::quote! {
        // Define the enum exectly as the user did
        #input
        // Create a constant to represent the variant count
        const POSTMASTER_PAYLOADS_VARIANT_COUNT: usize = #variant_count;
        // Create a type which postmaster uses to know what the payloads enum is called
        type POSTMASTER_PAYLOADS_ENUM = #enum_ident;
    };
    output.into()
}

#[proc_macro_attribute]
pub fn addresses(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemEnum);
    let variant_count: usize = get_variant_count(&input);
    let enum_ident = input.clone().ident;

    // // Ensure the enum provided is defined as pub, else postmaster won't be able to see it
    // if !matches!(input.vis, syn::Visibility::Public(_)) {
    //     return syn::Error::new_spanned(
    //         input.ident,
    //         "The `post_haste::addresses` macro must be used on a enum which is declared with `pub`",
    //     )
    //     .into_compile_error()
    //     .into();
    // }

    let output = quote::quote! {
        // Define the enum exectly as the user did
        #input
        // Create a constant to represent the variant count
        const POSTMASTER_ADDRESSES_VARIANT_COUNT: usize = #variant_count;
        // Create a type which postmaster uses to know what the payloads enum is called
        type POSTMASTER_ADDRESSES_ENUM = #enum_ident;
    };
    output.into()
}

fn get_variant_count(input: &syn::ItemEnum) -> usize {
    input.variants.iter().map(|_| 1).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
