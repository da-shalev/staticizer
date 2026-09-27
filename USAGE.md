# Usage

Register input values with `#[staticizer::register]` and read them in const code
through `staticizer::Records::<T>::ITEMS`. A constant sees the records of the crate
that evaluates it and of that crate's dependencies, including dependencies its code
never refers to.

## Setup

Add `staticizer` as a dependency; nothing is installed. When a compiler (`rustc`,
or `clippy-driver` under `cargo clippy`) expands `#[staticizer::register]`, the
macro installs Staticizer's hook in it, and Cargo builds the macro with your
project's toolchain. The toolchain needs the `rustc-dev` component.

On stable, add this to the application's `.cargo/config.toml` so the macro crate,
and only it, can use rustc's unstable APIs. It applies to `cargo build`,
`cargo clippy` and rust-analyzer:

```toml
[env]
RUSTC_BOOTSTRAP = "staticizer_macros"
```

The crate that evaluates `Records::ITEMS` must itself use `#[staticizer::register]`,
since that is what installs the hook in its compilation.

## Example: add values declared in two crates

Both crates depend on `staticizer`. The application also depends on a library named `totals`.

**`totals/src/lib.rs`** declares 20 and defines how to add all registered amounts during compilation:

```rust
pub struct Amount(pub u32);

#[staticizer::register]
static BASE: Amount = Amount(20);

pub const fn total() -> u32 {
    let amounts = staticizer::Records::<Amount>::ITEMS;
    let mut sum = 0;
    let mut i = 0;
    while i < amounts.len() {
        sum += amounts[i].0;
        i += 1;
    }
    sum
}
```

**The application's `src/main.rs`** declares another 22 and evaluates the total:

```rust
#[staticizer::register]
static EXTRA: totals::Amount = totals::Amount(22);

const TOTAL: u32 = totals::total();

fn main() {
    println!("Total: {TOTAL}");
}
```

Run `cargo run`. Output:

```text
Total: 42
```

`TOTAL` is evaluated in the application, so it adds the library's 20 and the
application's 22. Code compiled inside `totals` sees only the library's 20.

Change `Amount(22)` to `Amount(5)` and rebuild:

```text
Total: 25
```

Use the same pattern to construct a graph or schedule: collect typed declarations
with `Records::<T>::ITEMS` and transform them in const code.
