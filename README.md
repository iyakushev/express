# Express DSL

A simple DSL that handles arithmetic computation at run-time.
It supports named function dispatching with AST simplification at its build-time.


### Example

In order to integrate this feature you simply create the interpreter and pass it the context.
``` rust
use express_eval::{Interpreter, Context, NamedExpression};

// type NamedExpression<'e> = (&'e str, &'e str)

fn foo(formulas: &[NamedExpression]) -> Result<Interpreter> {
    let intrp = Interpreter::new(formulas, Context::new())?;
    // tick holds results for each formula;
    for tick in intrp {
        //...
    }
}
```

### Neet features

* Compile-time function evaluation (function type: `constant` | `pure`);
* Call duplication optimization (removes repeated function calls with same arguments);
* Reference result inline;

### Custom code

To create an executable function use macros: `runtime_callable`.
Optionally it accepts attribute `pure` which denotes given function as... well... pure. Meaning it has no side-effects and we can always expect the same output for the same tuple of arguments.
``` rust
// some::crate::path.rs
#[inline]
#[runtime_callable(pure)]
fn add_answer(lhs: f64) -> f64 {
    lhs + 42.0
}
// ... other code ...


// some::crate::consts.rs
const NICE: i64 = 69;
// ... other code ...
```

After that you need to add your library to the interpreter context. You may do this with a `use_library` macro.
``` rust
use some; // refering to the code above

fn intrp_setup() {
    let ctx = Context::new();
    use_library! {
        context ctx;
        library some::crate;  // holds common root path
        constants {
            consts::NICE;
            // ...
        }
        functions {
            path::add_answer;
            // ...
        }
    }
    // ... 
}
```

Then you may compute expressions like so: `NICE * add_answer(420)`. Note that libraries must be included before AST creation. Otherwise interpreter would fail to deduce fn dispatch. 

**Nice** implementation detail. If your function is `pure` and its arguments known at _"compile time"_ (literals or other pure functions) evaluation tree can be partially or completely optimized to some constant value. This expression is reduced at the optimization stage: `NICE * add_answer(add_answer(0))` => `5796`
