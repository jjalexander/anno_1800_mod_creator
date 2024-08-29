#[derive(Debug)]
pub(crate) enum State {
    Included,
    Excluded,
    ExcludedByAncestor,
    Forced,
    ForcedByAncestor,
}
