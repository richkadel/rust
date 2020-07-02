#include "rustllvm.h"
#include "llvm/ProfileData/Coverage/CoverageMapping.h"
#include "llvm/ProfileData/Coverage/CoverageMappingWriter.h"
#include "llvm/ProfileData/InstrProf.h"
#include "llvm/Support/FileSystem.h"
#include "llvm/Support/Path.h"
#include "llvm/ADT/ArrayRef.h"

#include <iostream>

using namespace llvm;

extern "C" SmallVectorTemplateBase<coverage::CounterMappingRegion>* LLVMRustCoverageNewSmallVectorCounterMappingRegion() {
  return new SmallVector<coverage::CounterMappingRegion, 32>();
}

extern "C" void LLVMRustCoverageDeleteSmallVectorCounterMappingRegion(SmallVectorTemplateBase<coverage::CounterMappingRegion>* Vector) {
  delete Vector;
}

extern "C" void LLVMRustCoverageFilenamesSectionToBuffer(RustStringRef BufferOut,
                                                      const LLVMValueRef* Filenames,
                                                      unsigned NumFilenames) {
  RawRustStringOstream OS(BufferOut);
  auto FilenamesWriter = coverage::CoverageFilenamesSectionWriter(
    makeArrayRef(Filenames, NumFilenames));
  FilenamesWriter.write(OS);
}

extern "C" void LLVMRustCoverageMappingToBuffer(
    RustStringRef BufferOut,
    const LLVMValueRef* VirtualFileMappings,
    unsigned NumVirtualFileMappings,
    const LLVMValueRef* Expressions,
    unsigned NumExpressions,
    SmallVectorTemplateBase<coverage::CounterMappingRegion>* MappingRegions) {
  RawRustStringOstream OS(BufferOut);
  auto CoverageMappingWriter = coverage::CoverageMappingWriter(
    makeArrayRef(VirtualFileMappings, NumVirtualFileMappings),
    makeArrayRef(Expressions, NumExpressions),
    makeMutableArrayRef(MappingRegions));
  CoverageMappingWriter.write(OS);
}

extern "C" uint32_t LLVMRustCoverageMappingVersion() {
  return coverage::CovMapVersion::CurrentVersion;
}

extern "C" void LLVMRustCoverageMappingVarName(RustStringRef NameOut) {
  auto name = getCoverageMappingVarName();
  RawRustStringOstream NameOS(NameOut);
  NameOS << name;
}

extern "C" void LLVMRustCoverageSectionName(LLVMModuleRef M,
                                            RustStringRef NameOut) {
  Triple TargetTriple(unwrap(M)->getTargetTriple());
  getInstrProfSectionName(IPSK_covmap, TargetTriple.getObjectFormat(),
                          /*AddSegmentInfo=*/false);
  auto name = getCoverageMappingVarName();
  RawRustStringOstream NameOS(NameOut);
  NameOS << name;
}

extern "C" void LLVMRustCoverageNormalizeFilename(const char* Filename,
                                                  RustStringRef NormalizedFilenameOut) {
  SmallString<256> Path(Filename);  // TODO(richkadel): REMOVE COMMENT... Note, this expects llvm "StringRef" type. Convert if not auto convertable.
  sys::fs::make_absolute(Path);
  sys::path::remove_dots(Path, /*remove_dot_dot=*/true);
  NormalizedFilenameOut = Path.str().str();
}

extern "C" void LLVMRustCoverageAddCounterRegion(
    SmallVectorTemplateBase<coverage::CounterMappingRegion>* MappingRegions,
    unsigned Index,
    unsigned FileID,
    unsigned LineStart,
    unsigned ColumnStart,
    unsigned LineEnd,
    unsigned ColumnEnd) {
  auto Counter = coverage::Counter::getCounter(Index);
  MappingRegions.push_back(coverage::CounterMappingRegion::makeRegion(
           Counter, FileID, LineStart,
           ColumnStart, LineEnd, ColumnEnd));
}
