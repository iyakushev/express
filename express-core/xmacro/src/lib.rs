mod uselib;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, parse_quote, spanned::Spanned, FnArg, Pat, ReturnType};
use uselib::Library;

fn parse_purity_attr(attr: TokenStream) -> Result<bool, syn::Error> {
    match syn::parse::<syn::Ident>(attr) {
        Ok(tt) => match tt.to_string().as_str() {
            "pure" => Ok(true),
            "" => Ok(false),
            _ => Err(syn::Error::new(
                tt.span(),
                "Macro accepts only one attribute 'pure'",
            )),
        },
        Err(_) => Ok(false),
    }
}

fn mangle_struct_name(name: syn::Ident) -> syn::Ident {
    format_ident!("__{}", name)
}

/// This is a special macro that qualifies given function
/// as a runtime acceptable. Note that function can't
/// have any internal mutable state.
/// # Example
/// ```ignore
/// # #[macro_use] extern crate exmac;
/// # use exmac::runtime_callable;
/// #[runtime_callable(pure)]
/// fn foo(input: f64) -> f64 {
///     input + 3.14f64
/// }
/// ```
/// This expands given function into runtime callable object:
/// ```ignore
/// use types::{Callable, Type};
/// #[allow(non_camel_case_types)]
/// struct _foo;
/// impl Callable for _foo{
///     fn call(&self, args: Box<[Type]>) -> Type {
///         let input: f64 = unsafe { args.get_unchecked(0usize).into() };
///         { input + 3.14f64 }.into()
///     }
/// }
/// ```
/// ## Safety:
/// `unsafe` block helps to remove unnecessary bounds checks which are preformed
/// at runtime before that.
#[proc_macro_attribute]
pub fn runtime_callable(attr: TokenStream, item: TokenStream) -> TokenStream {
    let is_pure = match parse_purity_attr(attr) {
        Ok(pure) => pure,
        Err(e) => return e.to_compile_error().into(),
    };
    let function: syn::ItemFn = syn::parse_macro_input!(item);
    let mut arguments: Vec<_> = Vec::new();
    let mut argcnt: usize = 0;
    for arg in function.sig.inputs.clone().into_iter() {
        if let FnArg::Typed(t) = arg.clone() {
            if let Pat::Ident(id) = *t.pat.clone() {
                let tp = t.ty;
                let q = quote! {
                    let #id : #tp = unsafe { args.get_unchecked(#argcnt).into() };
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
        argcnt += 1;
    }
    let fn_name = mangle_struct_name(function.sig.ident.clone());
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
        // use express::types::{Callable, Type};

        #[allow(dead_code)]
        #function

        #[allow(non_camel_case_types)]
        pub struct #fn_name;

        impl Callable for #fn_name {
            #( #attrs )*
            fn call(&self, args: &[Type]) -> Option<Type> {
                #( #arguments )*
                Some({ #( #stmts )* }?.into())
            }

            fn argcnt(&self) -> usize {
                #argcnt
            }

            fn is_pure(&self) -> bool {
                #is_pure
            }
        }
    }
    .into()
}

/// Resolves function name
#[proc_macro]
pub fn resolve_name(item: TokenStream) -> TokenStream {
    let name: syn::Ident = syn::parse_macro_input!(item);
    let resolved = format_ident!("__{}", name);
    quote! {
        #resolved
    }
    .into()
}

/// Registers functions and constants in the given interpreter context;
/// Macro uses a custom syntax to expose constants and functions.
/// ## Syntax
/// Macro consists of 4 sections in total with __constants__ and __functions__
/// being mutualy optional.
/// (they can be used both at once, only one of them but never without any of them)
/// : __context__ -- is a section where user passes a __Context__ object.
/// : __library__ -- this declares a common library root which is used in later sections.
/// : __constants__ -- registers constants.
/// : __functions__ -- registers functions.
///
/// ```ignore
/// use some_xpr_lib;
///
/// fn setup_interpreter() {
///     let mut ctx = Context::new();
///     // ...some other code...
///     use_library! {
///         context ctx;
///         library some_xpr_lib::subcrate;
///         constatns {
///             path::const::math::FOO;
///             path::const::lib::BAR;
///         }
///         functions {
///             other::path::book;
///             other::path::math::sin;
///         }
///     };
///     // ...other library regestration...
///     let intrp = Interpreter::new(ctx);
///     // ...
/// }
/// ```
/// For example, the resulting path of constant `FOO` is `some_xpr_lib::subcrate::path::const::math::FOO`.
#[proc_macro]
pub fn use_library(item: TokenStream) -> TokenStream {
    // let parsed = parse_macro_input!();
    let lib = parse_macro_input!(item as Library);
    let ctx = lib.ctx;
    let root = lib.root;
    let reg_constant: Vec<_> = lib
        .constants
        .values
        .into_iter()
        .map(|(name, path, target)| {
            // quote! { #ctx.register_constant(#name, #root #(::#path)?::#target); }
            if !path.path.segments.is_empty() {
                let mut full_path = root.clone();
                full_path.path.segments.extend(path.path.segments);
                quote! { #ctx.register_constant(#name, #full_path::#target); }
            } else {
                quote! { #ctx.register_constant(#name, #root::#target); }
            }
        })
        .collect();
    let reg_function: Vec<_> = lib
        .functions
        .values
        .into_iter()
        .map(|(name, path, target)| {
            let trgt = format_ident!("__{}", target);
            if !path.path.segments.is_empty() {
                let mut full_path = root.clone();
                full_path.path.segments.extend(path.path.segments);
                quote! { #ctx.register_function(#name, Box::new(#full_path::#trgt)); }
            } else {
                quote! { #ctx.register_function(#name, Box::new(#root::#trgt)); }
            }
        })
        .collect();
    quote! {
        #(#reg_constant);*
        #(#reg_function);*
    }
    .into()
}
