// extern crate failure;
extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

mod derive;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(PCDRecord)]
pub fn pcd_record_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive::f_pcd_record_derive(input)
        .unwrap_or_else(|err| err.to_compile_error().into())
        .into()
}
