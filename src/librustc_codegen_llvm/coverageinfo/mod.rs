use crate::builder::Builder;
use crate::common::CodegenCx;
use crate::llvm;
use libc::c_uint;
use log::debug;
use rustc_codegen_ssa::coverageinfo::map::*;
use rustc_codegen_ssa::traits::{CoverageInfoBuilderMethods, CoverageInfoMethods};
use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::small_c_str::SmallCStr;
use rustc_middle::ty::Instance;

use std::cell::RefCell;
use std::ffi::CString;

use crate::llvm::coverageinfo::CounterMappingRegion;

const COVMAP_VAR_ALIGN_BYTES: u32 = 8;

/// A context object for maintaining all state needed by the coverageinfo module.
pub struct CrateCoverageContext<'tcx> {
    // Coverage region data for each instrumented function identified by DefId.
    pub(crate) coverage_regions: RefCell<FxHashMap<Instance<'tcx>, FunctionCoverageRegions>>,
}

impl<'tcx> CrateCoverageContext<'tcx> {
    pub fn new() -> Self {
        Self { coverage_regions: Default::default() }
    }
}

fn real_filename(file: Lrc<SourceFile>) -> String {
    match file.name {
        Real(RealFileName(path)) => path.to_string_lossy(),
        _ => bug!("coverage mapping expected only real, named files")
    }
}

/// Generates and exports the Coverage Map.
// FIXME(richkadel): Complete all variations of generating and exporting the coverage map to LLVM.
// The current implementation is an initial foundation with basic capabilities (Counters, but not
// CounterExpressions, etc.).
pub fn finalize<'ll, 'tcx>(cx: &CodegenCx<'ll, 'tcx>) {
    let filename_ty = cx.type_ptr_to(self.type_i8());
    let mut filenames = Vec::<&llvm::Value>::new();

    let name_ref_i64 = cx.type_i64();
    let mapping_data_size_u32 = cx.type_i32();
    let func_hash_u64 = cx.type_i64();
    let function_record_ty = cx.type_struct(&[
        name_ref_i64,
        mapping_data_size_u32,
        func_hash_u64,
    ], /*packed=*/ false);

    // TODO(richkadel): If the instances can be in more than one file, move this into the loop, using a hashmap to ensure only one entry per filename
    filenames.push(cx.const_str(Symbol::intern("dummy_filename.rs"))); // TODO(richkadel): replace with real

    let coverage_mappings_buffer = llvm::build_byte_buffer(|coverage_mappings_buffer| {

//  CounterExpressionBuilder Builder;
//  ArrayRef<CounterExpression> getExpressions() const { return Expressions; }
//        let mut expressions = Vec::<&llvm::Value>::new();
        let mut counter_mapping_regions = makeSmallVectorCounterMappingRegion();
        let mut expressions = makeSmallVectorCounterExpression();

        let coverage_regions = &*cx.coverage_context().coverage_regions.borrow();
        for instance in coverage_regions.keys() {
            let coverageinfo = cx.tcx.coverageinfo(instance.def_id());
            let mangled_function_name = cx.tcx.symbol_name(caller_instance);
            debug_assert!(coverageinfo.num_counters > 0);
            debug!(
                "Generate coverage map for: {:?}, hash: {}, num_counters: {}",
                instance, coverageinfo.hash, coverageinfo.num_counters
            );

            let mut function_records = Vec::<&'ll llvm::Value>::new();
            let has_counters = false;

            let function_coverage_regions = &coverage_regions[instance];
            for (index, region) in function_coverage_regions.indexed_regions() {
                let loc = region.coverage_loc(cx);
                if filenames.len() == 0 {
                    let normalized_filename = CString::new(llvm::build_byte_buffer(|s| unsafe {
                        let filename = SmallCStr::new(loc.file.as_bytes());
                        llvm::LLVMRustCoverageNormalizeFilename(filename.as_c_str(), s);
                    })).expect("normalized_filename should be null-terminated");
                    debug!("normalized_filename: {}", normalized_filename);
                    filenames.push(real_filename(normalized_filename))
                }
                let file_id: u32 = 0; // TODO(richkadel): CHANGE THIS to lookup the file_id if
                                        // already there else put new filenames in IndexVec<u32,String maybe> and return next index
                match region.kind {
                    CoverageKind::Counter => {
                        has_counters = true;
                        debug!(
                            "  Counter {}, for byte_pos {}..{}, coverage_loc: {:?}",
                            index,
                            region.coverage_span.start_byte_pos,
                            region.coverage_span.end_byte_pos,
                            loc,
                        );
                        llvm::LLVMRustCoverageAddCounterRegion(
                            counter_mapping_regions,
                            index,
                            file_id,
                            loc.start_line,
                            loc.start_col,
                            loc.end_line,
                            loc.end_col,
                        );
                    }
                    CoverageKind::CounterExpression(lhs, op, rhs) => {
                        debug!(
                            "  CounterExpression {} = {} {:?} {}, for {}..{}",
                            index,
                            lhs,
                            op,
                            rhs,
                            region.coverage_span.start_byte_pos,
                            region.coverage_span.end_byte_pos
                        );
                        // FIXME(richkadel) -- reconsider naming of these functions,
                        // maybe the counter one should be: LLVMRustCoverageAddCounter
                        // and the SmallVector should be coverage_mapping_regions ???
                        // (Although that may not be consistent with Clang/LLVM source,
                        // and if so, I'm OK with "counter_mappings" for counter expressions, etc. too.
                        // Ultimately they are all generating counts, even if zero.)

                        // FIXME(richkadel): implement and call
                        //   llvm::LLVMRustCoverageAddCounterExpressionRegion(
                        //       expressions,
                        //       ...
                        //   );
                    },
                }
            }
            for unreachable in function_coverage_regions.unreachable_regions() {
                debug!(
                    "  Unreachable code region: {}..{}",
                    unreachable.start_byte_pos, unreachable.end_byte_pos
                );
                // FIXME(richkadel): implement and call
                //   llvm::LLVMRustCoverageAdd...something ... (
                //       counter_mapping_regions,
                //       ...
                //   );
            }

            if has_counters {

                let mut virtual_file_mapping = Vec::<u32>::new();
                // TODO(richkadel): I probably don't want to call it this way?
                gather_file_ids(&mut virtual_file_mapping);

                let old_len = coverage_mappings_buffer.len();
                llvm::LLVMRustCoverageMappingToBuffer(
                    coverage_mappings_buffer,
                    virtual_file_mapping.as_ptr(), // as *const &llvm::Value??? Or *const something else? //TODO(richkadel): remove comment
                    virtual_file_mapping.len() as c_uint,
                    expressions.as_ptr(),
                    expressions.len() as c_uint,
                    &counter_mapping_regions,
                );

                let name_ref = llvm::LLVMRustIndexedInstrProfComputeHash(mangled_function_name);
                let name_ref_val = cx.const_i64(name_ref);
                let mapping_data_size_val = cx.const_u32(coverage_mappings_buffer.len() - old_len);
                let func_hash_val = cx.const_u64(coverageinfo.hash);
                function_records.push(cx.const_struct(&[
                    name_ref_val, mapping_data_size_val, func_hash_val
                ], /*packed=*/ false));

                // FIXME(richkadel): For now, the clang coverage value "IsUsed" is assumed always true, so
                // nothing will be added to function_names.
                // if !is_used {
                //     function_names.push(
                //         // In C++: llvm::ConstantExpr::getBitCast(NamePtr, llvm::Type::getInt8PtrTy(Ctx)),
                //     )
                // }
            }
        }
    });

    // Create the deferred function function_records array
    // FIXME(richkadel): Will Rust compiled source ever have "deferred functions"?
    // let function_records_ty = cx.type_array(function_record_ty, function_records.len);
    let function_records_val = cx.const_array(function_record_ty, function_records);

    // let n_records_u32 = cx.type_i32();
    // let filenames_size_u32 = cx.type_i32();
    // let coverage_size_u32 = cx.type_i32();
    // let version_u32 = cx.type_i32();
    // let cov_data_header_ty = cx.type_struct(&[
    //     n_records_u32,
    //     filenames_size_u32,
    //     coverage_size_u32,
    //     version_u32,
    // ], /*packed=*/ false);

    // let filenames_ty = cx.type_array(filename_ty, filenames.len);
    let filenames_val = cx.const_array(filename_ty, &filenames[..]);
    let filenames_buffer = llvm::build_byte_buffer(|filenames_buffer| {
        llvm::LLVMRustCoverageFilenamesSectionToBuffer(
            filenames_buffer,
            filenames_val.as_ptr(),
            filenames_val.len() as c_uint,
        );
    });

    let filenames_size = filenames_buffer.len();
    let pad = buffer.len() % COVMAP_VAR_ALIGN_BYTES;
    coverage_mappings_buffer.append([0].repeat(pad));
    let coverage_size = coverage_mappings_buffer.len();
    let (filenames_and_coverage_mappings_val, _) = cx.const_str(Symbol::intern(filenames_buffer + coverage_mappings_buffer));

    let n_records_val = cx.const_u32(function_records.len());
    let filenames_size_val = cx.const_u32(filenames_size);
    let coverage_size_val = cx.const_u32(coverage_size);
    let version_val = cx.const_u32(unwrap(llvm::LLVMRustCoverageMappingVersion()));

    let cov_data_header_val = cx.const_struct(&[n_records_val, filenames_val, coverage_size_val, version_val], /*packed=*/ false);
    // Create the coverage data record
    // let cov_data_ty = cx.type_struct(&[
    //     cov_data_header_ty,
    //     // function_records_ty,
    //     cx.val_ty(function_records_val),
    //     cx.val_ty(filenames_and_coverage_mappings_val),
    // ], /*packed=*/ false);
    let cov_data_val = cx.const_struct([
        cov_data_header_val,
        function_records_val,
        filenames_and_coverage_mappings_val,
    ], /*packed=*/ false);

    let covmap_var_name = llvm::build_string(|s| unsafe {
        llvm::LLVMRustCoverageMappingVarName(s);
    })
    .expect("non-UTF8 coverage mapping var name");
    debug!("covmap var name: {}", covmap_var_name);

    let covmap_section_name = llvm::build_string(|s| unsafe {
        llvm::LLVMRustCoverageMappingSectionName(cx.llmod, s);
    })
    .expect("non-UTF8 covmap section name");
    debug!("covmap section name: {}", covmap_section_name);

    // TODO(richkadel): Remove this comment block after confirming the uncommented
    // version works.
    //
    // Alternative to LLVMAddGlobal?:
    //
    //   let g = cx.define_global(
    //       &covmap_var_name,
    //       cx.val_ty(cov_data_val),
    //   ).unwrap_or_else(|| {
    //       bug!("covmap section name symbol `{}` is already defined", covmap_var_name);
    //   });
    //   llvm::LLVMSetGlobalConstant(g, ffi::True);

    let llglobal = llvm::LLVMAddGlobal(
        cx.llmod,
        cx.val_ty(cov_data_val),
        covmap_var_name.to_string(),
    );
    llvm::LLVMSetInitializer(g, cov_data_val);
    llvm::LLVMRustSetLinkage(g, llvm::Linkage::InternalLinkage);
    let covmap_section_name = SmallCStr::new(covmap_section_name);
    llvm::LLVMRustSetSection(g, covmap_section_name.as_c_str());
    llvm::LLVMRustSetAlignment(g, COVMAP_VAR_ALIGN_BYTES);
    // TODO(richkadel): Make sure this Rust implementation doesn't require registering this global
    // as "used". Note clang has the following, to "make sure the data doesn't get deleted":
    //   CGM.addUsedGlobal(CovData);
    // This seems to do as much as `fn create_used_variable()`.
}

  /// Find the set of files we have regions for and assign IDs
  ///
  /// Fills \c Mapping with the virtual file mapping needed to write out
  /// coverage and collects the necessary file information to emit source and
  /// expansion regions.
  fn gather_file_ids(virtual_file_mapping: &Vec<u32>) {
    FileIDMapping.clear();

    llvm::SmallSet<FileID, 8> Visited;
    SmallVector<std::pair<SourceLocation, unsigned>, 8> FileLocs;
    for (const auto &Region : SourceRegions) {
      SourceLocation Loc = Region.getBeginLoc();
      FileID File = SM.getFileID(Loc);
      if (!Visited.insert(File).second)
        continue;

      // Do not map FileID's associated with system headers.
      if (SM.isInSystemHeader(SM.getSpellingLoc(Loc)))
        continue;

      unsigned Depth = 0;
      for (SourceLocation Parent = getIncludeOrExpansionLoc(Loc);
           Parent.isValid(); Parent = getIncludeOrExpansionLoc(Parent))
        ++Depth;
      FileLocs.push_back(std::make_pair(Loc, Depth));
    }
    llvm::stable_sort(FileLocs, llvm::less_second());

    for (const auto &FL : FileLocs) {
      SourceLocation Loc = FL.first;
      FileID SpellingFile = SM.getDecomposedSpellingLoc(Loc).first;
      auto Entry = SM.getFileEntryForID(SpellingFile);
      if (!Entry)
        continue;

      FileIDMapping[SM.getFileID(Loc)] = std::make_pair(virtual_file_mapping.len(), Loc);
      virtual_file_mapping.push(CVM.getFileID(Entry));
    }
  }
impl CoverageInfoMethods for CodegenCx<'ll, 'tcx> {
    fn coverageinfo_finalize(&self) {
        finalize(self)
    }
}

impl CoverageInfoBuilderMethods<'tcx> for Builder<'a, 'll, 'tcx> {
    fn new_counter_region(
        &mut self,
        instance: Instance<'tcx>,
        index: u32,
        start_byte_pos: u32,
        end_byte_pos: u32,
    ) {
        debug!(
            "adding counter to coverage map: instance={:?}, index={}, byte range {}..{}",
            instance, index, start_byte_pos, end_byte_pos,
        );
        let mut coverage_regions = self.coverage_context().coverage_regions.borrow_mut();
        coverage_regions
            .entry(instance)
            .or_insert_with(|| FunctionCoverageRegions::new())
            .new_counter(index, start_byte_pos, end_byte_pos);
    }

    fn new_counter_expression_region(
        &mut self,
        instance: Instance<'tcx>,
        index: u32,
        lhs: u32,
        op: CounterOp,
        rhs: u32,
        start_byte_pos: u32,
        end_byte_pos: u32,
    ) {
        debug!(
            "adding counter expression to coverage map: instance={:?}, index={}, {} {:?} {}, byte range {}..{}",
            instance, index, lhs, op, rhs, start_byte_pos, end_byte_pos,
        );
        let mut coverage_regions = self.coverage_context().coverage_regions.borrow_mut();
        coverage_regions
            .entry(instance)
            .or_insert_with(|| FunctionCoverageRegions::new())
            .new_counter_expression(index, lhs, op, rhs, start_byte_pos, end_byte_pos);
    }

    fn new_unreachable_region(
        &mut self,
        instance: Instance<'tcx>,
        start_byte_pos: u32,
        end_byte_pos: u32,
    ) {
        debug!(
            "adding unreachable code to coverage map: instance={:?}, byte range {}..{}",
            instance, start_byte_pos, end_byte_pos,
        );
        let mut coverage_regions = self.coverage_context().coverage_regions.borrow_mut();
        coverage_regions
            .entry(instance)
            .or_insert_with(|| FunctionCoverageRegions::new())
            .new_unreachable(start_byte_pos, end_byte_pos);
    }
}
