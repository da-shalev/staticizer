# Usage

Register a value with a `Record` impl marked `#[staticizer::register]`, and read every
registered value in const code through `Records::<T>::ITEMS`.

## Setup

Add `staticizer` as a dependency. The toolchain needs the `rustc-dev` component.

The crate that reads records must contain a `#[staticizer::register]` itself, since
that is what installs Staticizer's hook in its compiler.

On stable, add this to the application's `.cargo/config.toml`. It lets Staticizer's macro
crate, and no other crate, use unstable compiler APIs:

```toml
[env]
RUSTC_BOOTSTRAP = "staticizer_macros"
```

## Example

The library `totals` registers 20 and adds up every registered amount:

```rust
pub struct Amount(pub u32);

struct Base;

#[staticizer::register]
impl staticizer::Record<Amount> for Base {
    const ITEM: &'static Amount = &Amount(20);
}

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

The application registers 22 and reads the total:

```rust
struct Extra;

#[staticizer::register]
impl staticizer::Record<totals::Amount> for Extra {
    const ITEM: &'static totals::Amount = &totals::Amount(22);
}

const TOTAL: u32 = totals::total();

fn main() {
    println!("Total: {TOTAL}");
}
```

```text
Total: 42
```

## Where records are read

Records are read where the code is compiled, and a crate sees only its own records and
its dependencies'. The constant is evaluated in the application, so it sees both amounts.
Calling `total()` at runtime returns 20 instead: a plain function is compiled once, inside
`totals`, which cannot see the application.

To see every crate's records, read them in a constant, or in a generic function the
application instantiates.

## Records that depend on the application

A library can register a value built from a type the application defines. Reading the
records for that type builds each one for it:

```rust
#[staticizer::register]
impl<A: App> staticizer::Record<Handler, A> for Greeting {
    const ITEM: &'static Handler = &Handler(|| A::NAME);
}
```

## Order

Records come in the same order on every build, but the order means nothing. To order
them, give the record type a field to sort by.
