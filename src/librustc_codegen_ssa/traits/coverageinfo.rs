use super::BackendTypes;

pub trait CoverageInfoMethods: BackendTypes {
    fn coverageinfo_finalize(&self);
}
