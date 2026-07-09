//! Kernel memory subsystem
//! 
//! v0.0.6 starts the real memory-management groundwork. This module currently
//! reads the Limine memory map, logs physical memory regions, summarizes usable
//! RAM, and initializes a very small physical frame allocator.
//! 
//! This is not a heap allocator yet. Is is the foundation needed before page
//! table management, MMIO mapping, heap allocator and userspace memory can be
//! made general.

pub mod frame_allocator;

/// Whether to allocate a few frames during boot as a smoke test
/// 
/// This should normally stay disabled once the allocator is used by real
/// subsystems, because the early allocator does not support deallocation yet.
const RUN_FRAME_ALLOCATOR_SMOKE_TEST: bool = false;

/// Initializes early memory management
pub fn init() {
    crate::kprintln!("[NX][MEM] init begin");

    log_memory_map();

    match frame_allocator::init() {
        Ok(()) => {
            frame_allocator::log_state();

            if RUN_FRAME_ALLOCATOR_SMOKE_TEST {
                frame_allocator::debug_smoke_test();
            }
        }
        Err(error) => {
            crate::kprintln!("[NX][MEM] frame allocator init failed: {:?}", error);
        }
    }

    crate::kprintln!("[NX][MEM] init complete");
}

/// Logs the Limine memory map and a simple usable-memory summary
fn log_memory_map() {
    let Some(response) = crate::limine::memory_map_response() else {
        crate::kprintln!("[NX][MEM] memory map unavailable");
        return;
    };

    crate::kprintln!("[NX][MEM] memory map entries={}", response.entry_count);

    let mut usable_bytes = 0u64;
    let mut usable_regions = 0u64;

    for index in 0..response.entry_count as usize {
        let Some(entry) = crate::limine::memory_map_entry(index) else {
            crate::kprintln!("[NX][MEM] entry {} unavailable", index);
            continue;
        };

        let end = entry.base.saturating_add(entry.length);
        let kind_name = crate::limine::memory_kind_name(entry.kind);

        crate::kprintln!(
            "[NX][MEM] entry: {:02}: base={:#018x} end={:#018x} length={:#018x} kind={}",
            index,
            entry.base,
            end,
            entry.length,
            kind_name
        );

        if entry.kind == crate::limine::MEMORY_KIND_USABLE {
            usable_regions += 1;
            usable_bytes = usable_bytes.saturating_add(entry.length);
        }
    }

    crate::kprintln!(
        "[NX][MEM] usable regions={} usable_bytes={:#018x} usable_mib={}",
        usable_regions,
        usable_bytes,
        usable_bytes / 1024 / 1024
    );
}