pub use lang;
pub use types;
pub use xmacro;

/// Holds basic public API to the compiler and useful type declaraions
pub mod prelude {
    pub use types::{Callable, CallableType, Function, InterpreterContext, Type};
    pub use xmacro::{resolve_name, runtime_callable, use_library};
}
