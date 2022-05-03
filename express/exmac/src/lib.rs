use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, FnArg, Pat, ReturnType};
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
    let function: syn::ItemFn = syn::parse_macro_input!(item);
    let mut arguments: Vec<_> = Vec::new();
    for (idx, arg) in function.sig.inputs.clone().into_iter().enumerate() {
        if let FnArg::Typed(t) = arg.clone() {
            if let Pat::Ident(id) = *t.pat.clone() {
                let tp = t.ty;
                let q = quote! {
                    let #id : #tp = unsafe { args.get_unchecked(#idx).into(); };
                };
                arguments.push(q);
            } else {
                return syn::Error::new(
                    t.span(),
                    format!(
                        "This macro expects identifier as an argument name. Got: {:?}",
                        arg
                    ),
                )
                .to_compile_error()
                .into();
            }
        } else {
            return syn::Error::new(
                arg.span(),
                "This macro cannot be applied to functions that use Reciever types like `self`",
            )
            .to_compile_error()
            .into();
        }
    }
    let fn_name = format_ident!("_{}", function.sig.ident);
    if let ReturnType::Default = function.sig.output {
        return syn::Error::new(
            function.sig.output.span(),
            "Function must be explicit in its return type",
        )
        .to_compile_error()
        .into();
    }

    let stmts = function.block.stmts;
    quote! {
        #[allow(non_camel_case_types)]
        struct #fn_name;

        impl Callable for #fn_name {
            fn call(&self, args: &[Type]) -> Type {
                #( #arguments )*
                return {
                    #( #stmts )*
                }.into();
            }
        }
    }
    .into()
}
