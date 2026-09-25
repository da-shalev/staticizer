use base::Amount;

#[staticizer::register]
static LIBRARY: Amount = Amount(10);

#[test]
fn library_sees_its_own_and_base_records() {
    const TOTAL: u32 = base::total();
    assert_eq!(TOTAL, 1 + 10);
}
