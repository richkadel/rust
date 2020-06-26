use crate::common::CodegenCx;
use log::debug;
use rustc_codegen_ssa::traits::CoverageInfoMethods;
use rustc_hir::def_id::LOCAL_CRATE;

// FIXME(richkadel): This might not be where I put the coverage map generation, but it's a
// placeholder for now.
//
// I need to inject the coverage map into the LLVM IR. This is one possible place to do it.
//
// The code below is here, at least temporarily, to verify that the `CoverageRegion`s are
// available, to produce the coverage map.

/// Generates and exports the Coverage Map.
pub fn finalize(cx: &CodegenCx<'_, '_>) {
    let instances = &*cx.instances.borrow();
    for instance in instances.keys() {
        if instance.def_id().krate == LOCAL_CRATE {
            // FIXME(richkadel): Is this check `krate == LOCAL_CRATE` right?
            //
            // NOTE: I'm considering at least one alternative to this loop on `instances`,
            // But if I do go with this, make sure the check on LOCAL_CRATE works for all cases.
            //
            //
            // Without it, I was crashing on some of the instances, with:
            //
            //     src/librustc_metadata/rmeta/decoder.rs:1127:17: get_optimized_mir: missing MIR for `DefId(1:2867 ~ std[70cc]::io[0]::stdio[0]::_print[0])`
            //
            // This check avoids it, but still works for the very limited testing I've done so far.
            let coverageinfo = cx.tcx.coverageinfo(instance.def_id());
            if coverageinfo.num_counters > 0 {
                debug!(
                    "Generate coverage map for: {}, hash: {:?}, num_counters: {}\n{}",
                    instance,
                    coverageinfo.hash,
                    coverageinfo.num_counters,
                    {
                        let regions = &coverageinfo.coverage_regions;
                        (0..regions.len() as u32)
                            .map(|counter_index| {
                                let region = &regions[counter_index];
                                format!(
                                    "  counter_index {} byte range: {}..{}",
                                    counter_index, region.start_byte_pos, region.end_byte_pos
                                )
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                    }
                );
            }
        }
    }
}

impl CoverageInfoMethods for CodegenCx<'ll, 'tcx> {
    fn coverageinfo_finalize(&self) {
        finalize(self)
    }
}
