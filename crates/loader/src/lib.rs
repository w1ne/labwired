use anyhow::{anyhow, Context, Result};
use goblin::elf::program_header::PT_LOAD;
use goblin::elf::Elf;
use labwired_core::memory::ProgramImage;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};
use std::sync::Arc;
use std::rc::Rc;

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

pub struct SourceLocation {
    pub file: String,
    pub line: Option<u32>,
    pub function: Option<String>,
}

pub struct SymbolProvider {
    #[allow(dead_code)]
    data: Arc<Vec<u8>>,
    context: addr2line::Context<addr2line::gimli::EndianReader<addr2line::gimli::RunTimeEndian, Rc<[u8]>>>,
}

impl SymbolProvider {
    pub fn new(path: &Path) -> Result<Self> {
        let data = fs::read(path).with_context(|| format!("Failed to read ELF for symbols: {:?}", path))?;
        let data = Arc::new(data);
        
        let slice: &'static [u8] = unsafe { std::mem::transmute(&data[..]) };
        
        let object = object::File::parse(slice).context("Failed to parse ELF for symbols")?;
        let context = addr2line::Context::new(&object).context("Failed to create addr2line context")?;
        
        Ok(Self {
            data,
            context,
        })
    }

    pub fn lookup(&self, addr: u64) -> Option<SourceLocation> {
        let mut frames = match self.context.find_frames(addr) {
            addr2line::LookupResult::Output(Ok(frames)) => frames,
            _ => return None,
        };
        
        if let Ok(Some(frame)) = frames.next() {
            let file = frame.location.as_ref()
                .and_then(|l| l.file)
                .map(|f: &str| f.to_string());
            let line = frame.location.as_ref().and_then(|l| l.line);
            let function = frame.function.as_ref()
                .and_then(|f| f.demangle().ok())
                .map(|s: std::borrow::Cow<str>| s.into_owned());
            
            if let Some(f) = file {
                return Some(SourceLocation {
                    file: f,
                    line,
                    function,
                });
            }
        }
        None
    }
}
