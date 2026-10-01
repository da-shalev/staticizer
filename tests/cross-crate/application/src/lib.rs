use base::Amount;

struct Application;

#[staticizer::register]
impl staticizer::Record<Amount> for Application {
    const ITEM: &'static Amount = &Amount(100);
}

mod nested {
    use base::Amount;

    struct Nested;

    #[staticizer::register]
    impl staticizer::Record<Amount> for Nested {
        const ITEM: &'static Amount = &Amount(1000);
    }
}

#[test]
fn application_sees_every_crate_once() {
    const TOTAL: u32 = base::total();
    assert_eq!(TOTAL, 1 + 10 + 100 + 1000);
    assert_eq!(staticizer::Records::<Amount>::ITEMS.len(), 4);
}

struct App;

#[staticizer::register]
impl staticizer::Record<base::Share> for Application {
    const ITEM: &'static base::Share = &base::Share(3);
}

#[staticizer::register]
impl staticizer::Record<base::Share, App> for Application {
    const ITEM: &'static base::Share = &base::Share(7);
}

#[test]
fn records_are_read_for_the_named_app() {
    assert_eq!(staticizer::Records::<Amount, App>::ITEMS.len(), 0);
    const FOR_APP: u32 = base::shares::<App>();
    const FOR_UNIT: u32 = base::shares::<()>();
    assert_eq!((FOR_APP, FOR_UNIT), (50 + 7, 50 + 3));
}

#[test]
fn records_are_ordered_by_module_path() {
    let amounts: Vec<u32> = staticizer::Records::<Amount>::ITEMS
        .iter()
        .map(|amount| amount.0)
        .collect();
    assert_eq!(amounts, [1000, 100, 1, 10]);
}
