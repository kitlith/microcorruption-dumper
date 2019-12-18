mod utils;

use wasm_bindgen::prelude::*;
use faerie::{ArtifactBuilder, Data, DataType, Decl, artifact::DefinedDecl, SectionKind};
use goblin::{elf, container::{Ctx, Endian, Container}};
use scroll::{Pread, Pwrite};
use target_lexicon::{Architecture, BinaryFormat, Environment, OperatingSystem, Triple, Vendor};
use std::collections::BTreeMap;
use byteorder::{ByteOrder, LE};

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

    let entry_point = LE::read_u16(&memory[0xfffe..]).into();

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

    let mut elf: goblin::elf::Elf = bin.pread(0).map_err(|e: goblin::error::Error| e.to_string())?;

    // make it an executable
    elf.header.e_type = elf::header::ET_EXEC;
    elf.header.e_entry = entry_point;
    elf.header.e_flags = 0x00000112; // EXEC_P, HAS_SYMS, D_PAGED

    // build a ProgramHeader
    let pheader = elf.section_headers.iter()
        .find(|h| h.sh_flags & elf::section_header::SHF_ALLOC as u64 != 0)
        .map(|section| elf::ProgramHeader {
            p_type: elf::program_header::PT_LOAD,
            p_flags: 7, // rwx, probably // TODO
            p_offset: section.sh_offset,
            p_vaddr: 0,
            p_paddr: 0,
            p_filesz: section.sh_size,
            p_memsz: section.sh_size,
            p_align: 0x10000
        }).ok_or("missing load section")?;

    let ctx = Ctx::new(Container::Little, Endian::Little);

    // locate the ProgramHeader
    elf.header.e_phoff = bin.len() as u64; // end of file. TODO: alignment?
    elf.header.e_phentsize = elf::ProgramHeader::size(ctx) as u16;
    elf.header.e_phnum = 1;

    let header = elf.header;
    drop(elf);

    // write the updated header
    bin.pwrite_with(header, 0, ctx.le).map_err(|e| e.to_string())?;

    // write ProgramHeader
    let mut phbin = vec![0; header.e_phentsize as usize];
    phbin.pwrite_with(pheader, 0, ctx).map_err(|e| e.to_string())?;
    bin.extend(phbin);

    Ok(bin.into_boxed_slice())
}
