use express::types::{Callable, Function, Type};
use express::xmacro::runtime_callable;

/// Takes any **pure Callable** and calls init with a list of arguments.
fn init(arg: Function) -> Option<Function> {
    None
}
