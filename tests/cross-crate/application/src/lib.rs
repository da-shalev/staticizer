use base::Amount;

#[staticizer::register]
static APPLICATION: Amount = Amount(100);

mod nested {
    use base::Amount;

    #[staticizer::register]
    static NESTED: Amount = Amount(1000);
}

#[test]
fn application_sees_every_crate_once() {
    const TOTAL: u32 = base::total();
    assert_eq!(TOTAL, 1 + 10 + 100 + 1000);
    assert_eq!(staticizer::Records::<Amount>::ITEMS.len(), 4);
}

#[test]
fn records_are_ordered_by_module_path() {
    let amounts: Vec<u32> = staticizer::Records::<Amount>::ITEMS
        .iter()
        .map(|amount| amount.0)
        .collect();
    assert_eq!(amounts, [100, 1000, 1, 10]);
}
