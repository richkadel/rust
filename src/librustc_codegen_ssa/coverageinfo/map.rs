use rustc_data_structures::fx::FxHashMap;
use rustc_data_structures::sync::Lrc;
use rustc_span::source_map::{SourceFile, SourceFileAndLine, SourceMap};

use std::collections::hash_map;
use std::slice;

#[derive(Copy, Clone, Debug)]
pub enum CounterOp {
    Add,
    Subtract,
}

#[derive(Debug)]
pub enum CoverageKind {
    Counter,
    CounterExpression(u32, CounterOp, u32),
}

#[derive(Debug)]
pub struct CoverageSpan {
    pub start_byte_pos: u32,
    pub end_byte_pos: u32,
}

#[derive(Debug)]
pub struct CoverageRegion {
    pub kind: CoverageKind,
    pub coverage_span: CoverageSpan,
}

/// A source code region used with coverage information.
#[derive(Debug)]
pub struct CoverageLoc {
    /// Information about the original source file.
    pub file: Lrc<SourceFile>,
    /// The (1-based) line number of the region start.
    pub start_line: u32,
    /// The (1-based) column number of the region start.
    pub start_col: u32,
    /// The (1-based) line number of the region end.
    pub end_line: u32,
    /// The (1-based) column number of the region end.
    pub end_col: u32,
}

fn lookup_file_line_col(source_map: &SourceMap, byte_pos: usize) -> (Lrc<SourceFile>, u32, u32) {
    let found = source_map.lookup_line(byte_pos).expect("should find coverage region byte position in source");
    let file = found.sf;
    let line_pos = file.line_begin_pos(byte_pos);

    // Use 1-based indexing.
    let line = (found.line + 1) as u32;
    let col = (byte_pos - line_pos).to_u32() + 1;

    (file, line, col)
}

impl CoverageRegion {
    fn coverage_loc<'ll, 'tcx>(&self, cx: &CodegenCx<'ll, 'tcx>) -> CoverageLoc {
        let source_map = self.sess().source_map();
        let (file, start_line, start_col) = lookup_file_line_col(source_map, self.coverage_span.start_byte_pos);
        let (_, end_line, end_col) = lookup_file_line_col(source_map, self.coverage_span.end_byte_pos);
        CoverageLoc {
            file,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }
}

/// Collects all of the coverage regions associated with (a) injected counters, (b) counter
/// expressions (additions or subtraction), and (c) unreachable regions (always counted as zero),
/// for a given Function. Counters and counter expressions are indexed because they can be operands
/// in an expression.
///
/// Note, it's important to distinguish the `unreachable` region type from what LLVM's refers to as
/// a "gap region" (or "gap area"). A gap region is a code region within a counted region (either
/// counter or expression), but the line or lines in the gap region are not executable (such as
/// lines with only whitespace or comments). According to LLVM Code Coverage Mapping documentation,
/// "A count for a gap area is only used as the line execution count if there are no other regions
/// on a line."
pub struct FunctionCoverageRegions {
    indexed: FxHashMap<u32, CoverageRegion>,
    unreachable: Vec<CoverageSpan>,
}

impl FunctionCoverageRegions {
    pub fn new() -> Self {
        Self { indexed: FxHashMap::default(), unreachable: Default::default() }
    }

    pub fn new_counter(&mut self, index: u32, start_byte_pos: u32, end_byte_pos: u32) {
        self.indexed.insert(
            index,
            CoverageRegion {
                kind: CoverageKind::Counter,
                coverage_span: CoverageSpan { start_byte_pos, end_byte_pos },
            },
        );
    }

    pub fn new_counter_expression(
        &mut self,
        index: u32,
        lhs: u32,
        op: CounterOp,
        rhs: u32,
        start_byte_pos: u32,
        end_byte_pos: u32,
    ) {
        self.indexed.insert(
            index,
            CoverageRegion {
                kind: CoverageKind::CounterExpression(lhs, op, rhs),
                coverage_span: CoverageSpan { start_byte_pos, end_byte_pos },
            },
        );
    }

    pub fn new_unreachable(&mut self, start_byte_pos: u32, end_byte_pos: u32) {
        self.unreachable.push(CoverageSpan { start_byte_pos, end_byte_pos });
    }

    pub fn indexed_regions(&self) -> hash_map::Iter<'_, u32, CoverageRegion> {
        self.indexed.iter()
    }

    pub fn unreachable_regions(&self) -> slice::Iter<'_, CoverageSpan> {
        self.unreachable.iter()
    }
}
