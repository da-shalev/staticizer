pub struct Amount(pub u32);

#[staticizer::register]
static BASE: Amount = Amount(1);

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

#[test]
fn base_sees_only_its_own_records() {
    const TOTAL: u32 = total();
    assert_eq!(TOTAL, 1);
}
