use library::Amount;

#[staticizer::register]
static EXTRA: Amount = Amount(22);

mod nested {
    use library::Amount;

    #[staticizer::register]
    static NESTED: Amount = Amount(100);
}

#[test]
fn a_constant_sees_its_crate_and_dependency_records() {
    const TOTAL: u32 = library::total();
    assert_eq!(TOTAL, 142);
}

#[test]
fn a_dependency_sees_only_its_own_records() {
    assert_eq!(library::total_seen_by_library(), 20);
}

#[test]
fn records_are_ordered_by_module_path() {
    let amounts: Vec<u32> = staticizer::Records::<Amount>::ITEMS
        .iter()
        .map(|amount| amount.0)
        .collect();
    assert_eq!(amounts, [22, 100, 20]);
}
