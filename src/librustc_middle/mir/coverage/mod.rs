//! Metadata from source code coverage analysis and instrumentation.

use rustc_index::vec::IndexVec;

/// Coverage information associated with each function (MIR) instrumented with coverage counters, when
/// compiled with `-Zinstrument_coverage`. The query `tcx.coverageinfo(DefId)` computes these
/// values on demand (during code generation). This query is only valid after executing the MIR pass
/// `InstrumentCoverage`.
#[derive(Clone, RustcEncodable, RustcDecodable, Debug, HashStable)]
pub struct CoverageInfo {
    /// A hash value that can be used by the consumer of the coverage profile data to detect
    /// changes to the instrumented source of the associated MIR body (typically, for an
    /// individual function).
    pub hash: u64,

    /// The total number of coverage region counters added to the MIR `Body`.
    pub num_counters: u32,

    /// The start and end positions within a source file for the region of source code counted by
    /// the given counter index.
    pub coverage_regions: IndexVec<u32, CoverageRegion>,
}

/// Defines the region, within the source code, where a counter is optionally injected (if compiled
/// with `-Zinstrument_coverage`), to count the number of times this code region is executed.
#[derive(Clone, RustcEncodable, RustcDecodable, Debug, HashStable)]
pub struct CoverageRegion {
    /// The code region's starting position within the source code file.
    pub start_byte_pos: u32,

    /// The code region's ending position within the source code file.
    pub end_byte_pos: u32,
}

/// Positional arguments to `libcore::count_code_region()`
pub mod count_code_region_args {
    pub const COUNTER_INDEX: usize = 0;
    pub const START_BYTE_POS: usize = 1;
    pub const END_BYTE_POS: usize = 2;
}
