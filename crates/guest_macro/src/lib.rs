//! Simple library for serializing complex types to the wasmtime runtime using serde

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn export_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
	let data = parse_macro_input!(item as ItemFn);
	let name = &data.sig.ident;
	let extern_name = format_ident!("_wasm_guest_{}", name);
	let gen = {
		let mut argument_types = quote!();
		let mut call = quote!();
		for (i, arg) in data.sig.inputs.iter().enumerate() {
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
			#[no_mangle]
			pub unsafe extern "C" fn #extern_name(value: u64) -> u64 {
				let message:#argument_types = wasmtime_serde_guest::read_msg(value);
				wasmtime_serde_guest::write_msg(&#name(#call))
			}
		}
	};
	quote!(#gen #data).into()
}

struct FnImports {
	functions: Vec<syn::Signature>,
}

impl syn::parse::Parse for FnImports {
	fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
		let mut functions = vec![];
		while let Ok(f) = input.parse::<syn::Signature>() {
			functions.push(f);
			input.parse::<syn::Token![;]>()?;
		}
		Ok(FnImports { functions })
	}
}

#[proc_macro]
pub fn import_fn(input: TokenStream) -> TokenStream {
	let mut remote_fns = quote!();
	let mut local_fns = quote!();
	let ast = syn::parse_macro_input!(input as FnImports);
	for f in ast.functions.iter().cloned() {
		let remote_name = format_ident!("_wasm_host_{}", f.ident);
		let mut inputs = quote!();
		for item in &f.inputs {
			if let syn::FnArg::Typed(syn::PatType { pat: p, .. }) = item {
				if let syn::Pat::Ident(i) = p.as_ref() {
					inputs = quote!(#inputs #i,);
				} else {
					panic!()
				}
			} else {
				panic!()
			}
		}
		inputs = quote!((#inputs));
		local_fns = quote!(
			#local_fns
			#f {
				let ptr = wasmtime_serde_guest::write_msg(&#inputs);
				unsafe{wasmtime_serde_guest::read_msg(#remote_name(ptr))}
			}
		);
		remote_fns = quote!(
			#remote_fns
			fn #remote_name(ptr: u64) -> u64;
		);
	}
	quote! {
		#local_fns
		extern "C" {
			#remote_fns
		}
	}
	.into()
}
