use base::{Amount, Share};

struct Library;

#[staticizer::register]
impl staticizer::Record<Amount> for Library {
    const ITEM: &'static Amount = &Amount(10);
}

#[staticizer::register]
impl<A: 'static> staticizer::Record<Share, A> for Library {
    const ITEM: &'static Share = &Share(50);
}

#[test]
fn library_sees_its_own_and_base_records() {
    const TOTAL: u32 = base::total();
    assert_eq!(TOTAL, 1 + 10);
}

#[test]
fn generic_records_apply_to_every_app() {
    const SHARES: u32 = base::shares::<()>();
    assert_eq!(SHARES, 50);
}
