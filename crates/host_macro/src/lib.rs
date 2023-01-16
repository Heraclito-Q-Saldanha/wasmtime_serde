//! Simple library for serializing complex types to the wasmtime runtime using serde

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

struct FnHost {
	functions: Vec<syn::Ident>,
}

impl syn::parse::Parse for FnHost {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		let mut functions = vec![];
		let mut end = false;
		while let Ok(f) = input.parse::<syn::Ident>() {
			if end {
				panic!("comma")
			}
			functions.push(f);
			end = input.parse::<syn::Token![,]>().is_err();
		}
		Ok(Self { functions })
	}
}

#[proc_macro]
pub fn host_funcs(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as FnHost);
	let mut list = quote!();
	let len = input.functions.len();
	for name in input.functions {
		let name = format_ident!("_wasm_host_{}", name);
		let str_name: proc_macro2::TokenStream = format!(r#""{name}""#).parse().unwrap();
		list = quote!(#list  (#str_name, #name),);
	}
	quote!({
		const HOST_FUNC:[(&str, fn(&[u8]) -> Vec<u8>);#len] = [#list];
		&HOST_FUNC
	})
	.into()
}

#[proc_macro_attribute]
pub fn export_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let ast = parse_macro_input!(item as ItemFn);
	let name = &ast.sig.ident;
	let extern_name = format_ident!("_wasm_host_{}", name);
	let gen = {
		let mut argument_types = quote!();
		let mut call = quote!();
		for (i, arg) in ast.sig.inputs.iter().enumerate() {
			let i = syn::Index::from(i);
			call = quote!(#call message.#i,);
			if let syn::FnArg::Typed(t) = arg {
				let ty = &t.ty;
				argument_types = quote!(#argument_types #ty,);
			} else {
				panic!();
			}
		}
		argument_types = quote! { (#argument_types) };
		quote! {
			fn #extern_name(value: &[u8]) -> Vec<u8> {
				let message:#argument_types = wasmtime_serde_host::deserialize(value).unwrap();
				wasmtime_serde_host::serialize(&#name(#call)).unwrap()
			}
		}
	};
	quote!(#gen #ast).into()
}
