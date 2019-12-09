mod utils;

use wasm_bindgen::prelude::*;
use faerie::{ArtifactBuilder, Data, DataType, Decl, artifact::DefinedDecl, SectionKind};
use goblin::{elf, container::{Ctx, Endian, Container}};
use scroll::{Pread, Pwrite};
use target_lexicon::{Architecture, BinaryFormat, Environment, OperatingSystem, Triple, Vendor};
use std::collections::BTreeMap;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// extern {
//     fn alert(s: &str);
// }

#[wasm_bindgen]
pub fn gen_elf(name: &str, memory: Box<[u8]>, symbols: &JsValue) -> Result<Box<[u8]>, JsValue> {
    let symbols: BTreeMap<String, u64> = symbols.into_serde().map_err(|_| "Bad symbol input")?;
    let target = Triple {
        architecture: Architecture::Msp430,
        vendor: Vendor::Unknown,
        operating_system: OperatingSystem::None_,
        environment: Environment::Unknown,
        binary_format: BinaryFormat::Elf
    };

    let mut artifact = ArtifactBuilder::new(target)
        .name(name.to_string())
        .library(false)
        .finish();

    let section = Decl::section(SectionKind::Text)
        .with_datatype(DataType::Bytes)
        .with_writable(true)
        .with_executable(true)
        .with_loaded(true);

    artifact.declare("flash", Decl::Defined(DefinedDecl::Section(section))).map_err(|e| e.to_string())?;
    artifact.define_with_symbols("flash", Data::Blob(memory.into_vec()), symbols).map_err(|e| e.to_string())?;

    let mut bin = artifact.emit().map_err(|e| e.to_string())?;

    // Now let's modify the output of that to create a real executable, so that it works with Ghidra.

    // read header
    let ctx = Ctx::new(Container::Little, Endian::Little);
    let mut header: elf::Header = bin.pread_with(0, ctx.le).map_err(|e: goblin::error::Error| e.to_string())?;

    // make it an executable
    header.e_type = elf::header::ET_EXEC;
    header.e_entry = 0x4400;
    header.e_flags = 0x00000112; // EXEC_P, HAS_SYMS, D_PAGED

    // read section header
    let sheader: elf::SectionHeader = bin.pread_with((header.e_shoff + (header.e_shentsize as u64 * 3)) as usize, ctx).map_err(|e: goblin::error::Error| e.to_string())?;

    // build a ProgramHeader
    let mut pheader: elf::ProgramHeader = elf::ProgramHeader::new();
    pheader.read(); pheader.write(); pheader.executable(); // rwx
    pheader.p_offset = sheader.sh_offset;
    pheader.p_vaddr = 0;
    pheader.p_paddr = 0;
    pheader.p_filesz = sheader.sh_size;
    pheader.p_memsz = sheader.sh_size;

    // locate the ProgramHeader
    header.e_phoff = bin.len() as u64; // end of file. TODO: alignment?
    header.e_phentsize = elf::ProgramHeader::size(ctx) as u16;
    header.e_phnum = 1;

    // write ProgramHeader
    let mut phbin = vec![0; header.e_phentsize as usize];
    phbin.pwrite_with(pheader, 0, ctx).map_err(|e| e.to_string())?;
    bin.extend(phbin);

    // write the updated header
    bin.pwrite_with(header, 0, ctx.le).map_err(|e| e.to_string())?;

    Ok(bin.into_boxed_slice())
}
