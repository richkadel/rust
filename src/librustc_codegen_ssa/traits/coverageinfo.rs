use crate::coverageinfo::CounterOp;
//use crate::coverageinfo::map::FunctionCoverageRegions;
use super::BackendTypes;
use rustc_middle::ty::Instance;

pub trait CoverageInfoMethods: BackendTypes {
    fn coverageinfo_finalize(&self);
}

pub trait CoverageInfoBuilderMethods<'tcx>: BackendTypes {
    // fn coverage_regions(&self, instance: Instance<'tcx>) -> Option<&'tcx FunctionCoverageRegions>;

    fn new_counter_region(
        &mut self,
        instance: Instance<'tcx>,
        index: u32,
        start_byte_pos: u32,
        end_byte_pos: u32,
    );

    fn new_counter_expression_region(
        &mut self,
        instance: Instance<'tcx>,
        index: u32,
        lhs: u32,
        op: CounterOp,
        rhs: u32,
        start_byte_pos: u32,
        end_byte_pos: u32,
    );

    fn new_unreachable_region(
        &mut self,
        instance: Instance<'tcx>,
        start_byte_pos: u32,
        end_byte_pos: u32,
    );
}
