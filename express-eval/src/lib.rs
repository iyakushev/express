pub mod ctx;
pub mod formula;
pub mod interp;
pub mod ir;

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
///         constatns:
///             path::const::math::FOO;
///             path::const::lib::BAR;
///         functions:
///             other::path::book;
///             other::path::math::sin;
///     };
///     // ...other library regestration...
///     let intrp = Interpreter::new(ctx);
///     // ...
/// }
/// ```
/// For example, the resulting path of constant `FOO` is `some_xpr_lib::subcrate::path::const::math::FOO`.
#[macro_export]
macro_rules! use_library {
    (context $ctx: expr;
     library $lib_root: ident $(::$rinner: ident)*;
     constants: $( $cname: ident $(::$cinner: ident)* );* ;
     functions: $( $fname: ident $(::$finner: ident)* );* ;) => {{

        // Asserts that the type of the $ctx expression is ctx::Context
        let mut _ctx: ctx::Context = $ctx;
        use_library!(context _ctx; library $lib_root$($rinner)*; constants: $( $cname$($cinner)* );* ;);
        use_library!(context _ctx; library $lib_root$($rinner)*; functions: $( $fname$($finner)* );* ;);
    }};

    (context $ctx: expr;
     library $lib_root: ident $(::$rinner: ident)*;
     constants: $( $cname: ident $(::$cinner: ident)* );* ;) => {{
        $(
            let _: ctx::Context = $ctx;
            let value = $lib_root::$cname$(::$cinner)*;
            let mut name = stringify!($(::$cinner)*).split("::").last().unwrap().to_string();
            remove_whitespace(&mut name);
            $ctx.register_constant(name.as_str(), value);
        )*
    }};

    (context $ctx: expr;
     library $lib_root: ident $(::$rinner: ident)*;
     functions: $( $fname: ident $(::$finner: ident)* );* ;) => {{
        $(
            let _: ctx::Context = $ctx;
            let value = $lib_root::$fname$(::$finner)*;
            let mut name = stringify!($(::$finner)*).split("::").last().unwrap().to_string();
            remove_whitespace(&mut name);
            $ctx.register_function(name.as_str(), value)
        )*
    }};
}

macro_rules! prefixed_import {
    ($prefix: ident $(::$inner: ident)* : $($name: ident)+) => {
        $($prefix$(::$inner)*::$name;)*
    };
}

#[allow(dead_code)]
fn remove_whitespace(s: &mut String) {
    s.retain(|c| !c.is_whitespace());
}

#[cfg(test)]
mod test {
    use crate::ctx::Context;

    use super::*;
    use express::types::{Callable, Type};
    use express::xmacro::runtime_callable;
    use express_std;

    #[runtime_callable]
    fn foo() -> Option<f64> {
        Some(3.14)
    }

    #[test]
    fn test_uselib_macro() {
        let mut ctx = Context::new();
        use_library! {
            context ctx;
            library express_std;
            constants:
                math::PI;
                math::EPS;
            // functions:
            //     math::log::log;
        };
        assert!(!ctx.ns_const.is_empty());
        assert_eq!(*ctx.ns_const.get("PI").unwrap(), express_std::math::PI);
        assert_eq!(*ctx.ns_const.get("EPS").unwrap(), express_std::math::EPS);
    }

    #[test]
    fn test_prefixed_import() {
        let mut ctx = Context::new();
        use_library! {
            context ctx;
            library express_std;
            constants:
                //math::{PI, EPS};  // add groupping
                math::PI;
        }
    }
}
