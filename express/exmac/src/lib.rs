use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, FnArg, Pat};
use types::{Callable, Type};
/// This is a special macro that qualifies given function
/// as a runtime acceptable. Note that the function can't
/// have any internal mutable state.
/// # Example
/// ```rust
/// use express::runtime_callable;
/// #[runtime_callable]
/// fn foo(input: String) -> f64 { ... }
/// ```
#[proc_macro_attribute]
pub fn runtime_callable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function: syn::ItemFn = syn::parse(item).expect("Failed to parse input token stream");
    let mut arguments: Vec<_> = Vec::new();
    for (idx, arg) in function.sig.inputs.clone().into_iter().enumerate() {
        if let FnArg::Typed(t) = arg {
            if let Pat::Ident(id) = *t.pat.clone() {
                let q = quote! {
                let #id: #t.ty = unsafe { args.get_unchecked(#idx).into();
                }; };
                arguments.push(q);
            };
            syn::Error::new(
                function.span(),
                "This macro expects identifier as the argument name",
            )
            .to_compile_error();
        }
        syn::Error::new(
            function.span(),
            "This macro cannot be applied to functions that use `self`",
        )
        .to_compile_error();
    }
    let fn_name = format_ident!("_{}", function.sig.ident);
    let stmts = function.block.stmts;
    let callable = quote! {
        struct #fn_name;

        impl Callable for #fn_name {
            fn call(&self, args: &[Type]) -> Type {
                #( #arguments )*
                #( #stmts )*
            }
        }
    };
    callable.into()
}
