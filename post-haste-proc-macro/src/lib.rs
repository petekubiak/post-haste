use proc_macro::TokenStream;

/// This macro should be invoked by the user on the payloads enum. The enum declaration
/// must not be inside a function. This macro creates variables which are used by
/// `init_postmaster!()`, so `post_haste::addresses`, `post_haste::payloads` and `init_postmaster!()`
/// should all be invoked in the same scope.
/// This macro exports:
/// - type POSTMASTER_PAYLOADS_ENUM
#[proc_macro_attribute]
pub fn payloads(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemEnum);
    let enum_ident = input.clone().ident;

    let output = quote::quote! {
        // Define the enum exectly as the user did
        #input
        // Create a type which postmaster uses to know what the payloads enum is called
        type POSTMASTER_PAYLOADS_ENUM = #enum_ident;
    };
    output.into()
}

/// This macro should be invoked by the user on the addresses enum. The enum declaration
/// must not be inside a function. This macro creates variables which are used by
/// `init_postmaster!()`, so `post_haste::addresses`, `post_haste::payloads` and `init_postmaster!()`
/// should all be invoked in the same scope.
/// This macro exports:
/// - const POSTMASTER_ADDRESSES_VARIANT_COUNT: usize
/// - type POSTMASTER_ADDRESSES_ENUM
#[proc_macro_attribute]
pub fn addresses(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemEnum);
    let variant_count: usize = get_variant_count(&input);
    let enum_ident = input.clone().ident;

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

/// Used to get the number of variants on an enum, using the syn library
fn get_variant_count(input: &syn::ItemEnum) -> usize {
    input.variants.iter().count()
}
