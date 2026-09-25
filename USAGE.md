# Usage

Register input values with `#[staticizer::register]` and read them in const code
through `staticizer::Records::<T>::ITEMS`. A constant sees the records of the crate
that evaluates it and of that crate's dependencies. Rust only loads dependencies the
code refers to, so name a dependency that is used only for its records with
`use dependency as _;`.

## Setup

Install the compiler wrapper from the Staticizer checkout:

```sh
cargo +nightly install --path . --locked --force
```

Add this to the application's `.cargo/config.toml`:

```toml
[build]
rustc-wrapper = "staticizer"
```

The wrapper must be on `PATH`. Use the same nightly for Staticizer and the
application, with `rustc-dev` and `llvm-tools` installed. Inside
`nix develop`, omit `+nightly`.

After updating the wrapper, run `cargo clean` in the application to rebuild dependency metadata; Cargo does not track changes to the wrapper executable.

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

Run `cargo +nightly run`. Output:

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
