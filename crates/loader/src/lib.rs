use anyhow::{anyhow, Context, Result};
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::Elf;
use labwired_core::memory::ProgramImage;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

pub fn load_elf(path: &Path) -> Result<ProgramImage> {
    let buffer = fs::read(path).with_context(|| format!("Failed to read ELF file: {:?}", path))?;

    let elf = Elf::parse(&buffer).context("Failed to parse ELF binary")?;

    info!("ELF Entry Point: {:#x}", elf.entry);

    let mut program_image = ProgramImage::new(elf.entry);

    for ph in elf.program_headers {
        if ph.p_type == PT_LOAD {
            // We only care about loadable segments
            let start_addr = ph.p_paddr; // Physical address (LMA) is usually what we want for flash programming
            let size = ph.p_filesz as usize;
            let offset = ph.p_offset as usize;

            if size == 0 {
                continue;
            }

            debug!(
                "Found Loadable Segment: Addr={:#x}, Size={} bytes, Offset={:#x}",
                start_addr, size, offset
            );

            if offset + size > buffer.len() {
                return Err(anyhow!("Segment out of bounds in ELF file"));
            }

            let segment_data = buffer[offset..offset + size].to_vec();
            program_image.add_segment(start_addr, segment_data);
        }
    }

    if program_image.segments.is_empty() {
        warn!("No loadable segments found in ELF file");
    }

    Ok(program_image)
}
