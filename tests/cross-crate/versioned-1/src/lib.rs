use base::Version;

struct Release;

#[staticizer::register]
impl staticizer::Record<Version> for Release {
    const ITEM: &'static Version = &Version(1);
}
