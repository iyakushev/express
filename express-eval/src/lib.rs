pub mod ctx;
pub mod dag;
pub mod formula;
pub mod interp;
pub mod ir;

#[cfg(test)]
mod test {
    use crate::ctx::Context;
    use express::types::{Callable, InterpreterContext, Type};
    use express::xmacro::{runtime_callable, use_library};
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
            library express_std::math;
            constants {
                PI;
                EPS;
            }
            functions {
                log;
                ln;
            }
        };

        assert!(!ctx.ns_const.is_empty());
        assert_eq!(ctx.find_constant("PI").unwrap(), express_std::math::PI);
        assert_eq!(ctx.find_constant("EPS").unwrap(), express_std::math::EPS);

        assert!(!ctx.ns_fn.is_empty());
        assert_eq!(
            f64::from(
                ctx.find_function("ln")
                    .unwrap()
                    .call(&[Type::Number(2.0)])
                    .unwrap()
            ),
            express_std::math::ln(2.0).unwrap()
        )
    }

    //#[test]
    //fn test_prefixed_import() {
    //    let mut ctx = Context::new();
    //    use_library! {
    //        context ctx;
    //        library express_std;
    //        functions {
    //            math::log::log;
    //        }
    //    }
    //}
}
