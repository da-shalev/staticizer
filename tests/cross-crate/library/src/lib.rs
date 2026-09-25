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

#[inline(never)]
pub fn total_seen_by_library() -> u32 {
    const TOTAL: u32 = total();
    TOTAL
}
