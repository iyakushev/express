use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, FnArg, Pat, ReturnType};

/// This is a special macro that qualifies given function
/// as a runtime acceptable. Note that function can't
/// have any internal mutable state.
/// # Example
/// ```ignore
/// # #[macro_use] extern crate exmac;
/// # use exmac::runtime_callable;
/// #[runtime_callable]
/// fn foo(input: f64) -> f64 {
///     input + 3.14f64
/// }
/// ```
/// This expands given function into runtime callable object:
/// ```ignore
/// use types::{Callable, Type, FN_REGISTRY};
/// #[allow(non_camel_case_types)]
/// struct _foo;
/// impl Callable for _foo{
///     fn call(&self, args: Box<[Type]>) -> Type {
///         let input: f64 = unsafe { args.get_unchecked(0usize).into() };
///         { input + 3.14f64 }.into()
///     }
/// }
/// ```
/// ## Limitations
/// Note, each runtime_callable fn must be defined in a separate file.
/// ## Safety:
/// `unsafe` block helps to remove unnecessary bounds checks which are preformed
/// at runtime before that.
#[proc_macro_attribute]
pub fn runtime_callable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function: syn::ItemFn = syn::parse_macro_input!(item);
    let mut arguments: Vec<_> = Vec::new();
    for (idx, arg) in function.sig.inputs.clone().into_iter().enumerate() {
        if let FnArg::Typed(t) = arg.clone() {
            if let Pat::Ident(id) = *t.pat.clone() {
                let tp = t.ty;
                let q = quote! {
                    let #id : #tp = unsafe { args.get_unchecked(#idx).into() };
                };
                arguments.push(q);
            } else {
                return syn::Error::new(
                    t.span(),
                    "This macro expects identifier as an argument name.",
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
    let fn_name = format_ident!("{}_xprs", function.sig.ident);
    let attrs = function.attrs.clone();
    if let ReturnType::Default = function.sig.output {
        return syn::Error::new(
            function.sig.output.span(),
            "Function must be explicit in its return type",
        )
        .to_compile_error()
        .into();
    }

    let stmts = function.block.stmts.clone();
    quote! {
        use express::types::{Callable, Type, FN_REGISTRY};

        #function

        #[allow(non_camel_case_types)]
        pub struct #fn_name;

        impl Callable for #fn_name {
            #( #attrs )*
            fn call(&self, args: &[Type]) -> Option<Type> {
                #( #arguments )*
                Some({ #( #stmts )* }?.into())
            }
        }
    }
    .into()
}

#[proc_macro]
pub fn resolve_name(item: TokenStream) -> TokenStream {
    let name: syn::Ident = syn::parse_macro_input!(item);
    let resolved = format_ident!("{}_xprs", name);
    quote! {
        #resolved
    }
    .into()
}
