pub struct Amount(pub u32);

pub struct Version(pub u32);

struct Base;

#[staticizer::register]
impl staticizer::Record<Amount> for Base {
    const ITEM: &'static Amount = &Amount(1);
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

pub struct Share(pub u32);

/// Generic over `A`, so the crate that names `A` evaluates the records.
pub const fn shares<A: 'static>() -> u32 {
    let shares = staticizer::Records::<Share, A>::ITEMS;
    let mut sum = 0;
    let mut i = 0;
    while i < shares.len() {
        sum += shares[i].0;
        i += 1;
    }
    sum
}

#[test]
fn base_sees_only_its_own_records() {
    const TOTAL: u32 = total();
    assert_eq!(TOTAL, 1);
}
