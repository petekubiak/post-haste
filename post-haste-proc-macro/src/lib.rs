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
    let enum_ident = input.clone().ident;

    let output = quote::quote! {
        // Define the enum exectly as the user did
        #input
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
