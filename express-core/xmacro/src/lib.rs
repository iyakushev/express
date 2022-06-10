mod uselib;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, FnArg, Pat, ReturnType};
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

fn extract_type_from_option(ty: &syn::Type) -> Option<&syn::Type> {
    use syn::{GenericArgument, Path, PathArguments, PathSegment};

    fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
        match *ty {
            syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    // TODO store (with lazy static) the vec of string
    // TODO maybe optimization, reverse the order of segments
    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| &idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
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
    let fn_src_name = function.sig.ident.clone();
    let attrs = function.attrs.clone();
    let result_type = if let ReturnType::Type(_, ref ret_t) = function.sig.output {
        extract_type_from_option(&*ret_t)
    } else {
        return syn::Error::new(
            function.sig.output.span(),
            "Function must be explicit in its return type",
        )
        .to_compile_error()
        .into();
    };
    let stmts = function.block.stmts.clone();

    let call_ret_stmt = if let Some(_) = result_type {
        quote! { Some( {#( #stmts )*}?.into() ) }
    } else {
        quote! { Some( {#( #stmts )*}.into() ) }
    };

    quote! {
        // use express::types::{Callable, Type};

        #[allow(dead_code)]
        #function

        #[allow(non_camel_case_types)]
        pub struct #fn_name;

        impl Callable for #fn_name {
            #( #attrs )*
            fn call(&mut self, args: &[Type]) -> Option<Type> {
                #( #arguments )*
                #call_ret_stmt
            }

            fn init(&mut self, args: &[Type], ctx: &dyn InterpreterContext) {}

            #[inline(always)]
            fn name(&self) -> &str {
                stringify!(#fn_src_name)
            }

            #[inline(always)]
            fn argcnt(&self) -> usize {
                #argcnt
            }

            #[inline(always)]
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
    let resolved = mangle_struct_name(name);
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
            let trgt = mangle_struct_name(target);
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
