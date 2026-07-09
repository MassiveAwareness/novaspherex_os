//! Early physical frame allocator
//! 
//! This allocator consumes `Usable` regions from the Limine memory map and hands
//! out 4 KiB physical frames.
//! 
//! This is intentionally simple:
//! - no deallocation
//! - no bitmap
//! - no buddy allocator
//! - no head dependency
//! - no support for reclaiming bootloader memory yet
//! 
//! It is a bootstrap allocator used to prove that the kernel can safely discover
//! and reserve physical RAM frames. Later, this should evolve into a real frame
//! allocator backed by memory map classification and proper bookkeping.

#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

/// Size of one physical frame
pub const FRAME_SIZE: u64 = 4096;

/// Avoid allocating from the first MiB during bring-up
/// 
/// Limine should already classify memory correctly, but avoiding low memory
/// keeps up away from legacy firmware and platform-reserved areas while the
/// allocator is still primitive.
const MIN_ALLOCATABLE_PHYSICAL: u64 = 0x0010_0000;

/// Whether the frame allocator has been initialized
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Current Limine memory map entry index used by the bump allocator
static CURRENT_ENTRY_INDEX: AtomicUsize = AtomicUsize::new(0);

/// Next physical frame candicate
static NEXT_FRAME: AtomicU64 = AtomicU64::new(0);

/// Total usable bytes discovered at initialization time
static TOTAL_USABLE_BYTES: AtomicU64 = AtomicU64::new(0);

/// Total usable 4 KiB frames discovered at initialization time
static TOTAL_USABLE_FRAMES: AtomicU64 = AtomicU64::new(0);

/// Number of frames handed out by this allocator
static ALLOCATED_FRAMES: AtomicU64 = AtomicU64::new(0);

/// A single physical frame
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PhysicalFrame {
    /// Physical start address of the frame
    pub start: u64
}

/// Errors returned by the early frame allocator
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameAllocatorError {
    /// Limine did not provide a memory map
    NoMemoryMap,

    /// No usable frame was found
    NoUsableFrame
}

/// Initializes the early physical frame allocator
pub fn init() -> Result<(), FrameAllocatorError> {
    let Some(response) = crate::limine::memory_map_response() else {
        return Err(FrameAllocatorError::NoMemoryMap);
    };

    let mut total_usable_bytes = 0u64;
    let mut total_usable_frames = 0u64;

    let mut first_entry_index = None;
    let mut first_frame = 0u64;

    for index in 0..response.entry_count as usize {
        let Some(entry) = crate::limine::memory_map_entry(index) else {
            continue;
        };

        if entry.kind != crate::limine::MEMORY_KIND_USABLE {
            continue;
        }

        let start = align_up_4k(entry.base.max(MIN_ALLOCATABLE_PHYSICAL));
        let end = align_down_4k(entry.base.saturating_add(entry.length));

        if end <= start {
            continue;
        }

        let region_bytes = end - start;
        let region_frames = region_bytes / FRAME_SIZE;

        total_usable_bytes = total_usable_bytes.saturating_add(region_bytes);
        total_usable_frames = total_usable_frames.saturating_add(region_frames);

        if first_entry_index.is_none() {
            first_entry_index = Some(index);
            first_frame = start;
        }
    }

    let Some(index) = first_entry_index else {
        return Err(FrameAllocatorError::NoUsableFrame);
    };

    CURRENT_ENTRY_INDEX.store(index, Ordering::Relaxed);
    NEXT_FRAME.store(first_frame, Ordering::Relaxed);
    TOTAL_USABLE_BYTES.store(total_usable_bytes, Ordering::Relaxed);
    TOTAL_USABLE_FRAMES.store(total_usable_frames, Ordering::Relaxed);
    ALLOCATED_FRAMES.store(0, Ordering::Relaxed);
    INITIALIZED.store(true, Ordering::Relaxed);

    crate::kprintln!(
        "[NX][FRAME] initialized first_entry={} first_frame={:#018x}",
        index,
        first_frame
    );

    Ok(())
}

/// Allocates one 4 KiB physical frame
/// 
/// This function currently implements a simple memory-map-backend bump allocator.
/// Frames are never freed.
pub fn allocate_frame() -> Option<PhysicalFrame> {
    if !INITIALIZED.load(Ordering::Relaxed) {
        return None;
    }

    let response = crate::limine::memory_map_response()?;

    let mut index = CURRENT_ENTRY_INDEX.load(Ordering::Relaxed);
    let mut next = NEXT_FRAME.load(Ordering::Relaxed);

    while index < response.entry_count as usize {
        let Some(entry) = crate::limine::memory_map_entry(index) else {
            index += 1;
            continue;
        };

        if entry.kind != crate::limine::MEMORY_KIND_USABLE {
            index += 1;
            next = 0;
            continue;
        }

        let start = align_up_4k(entry.base.max(MIN_ALLOCATABLE_PHYSICAL));
        let end = align_down_4k(entry.base.saturating_add(entry.length));

        if next < start {
            next = start;
        }

        if next.saturating_add(FRAME_SIZE) <= end {
            let frame = PhysicalFrame {
                start: next
            };

            NEXT_FRAME.store(next + FRAME_SIZE, Ordering::Relaxed);
            CURRENT_ENTRY_INDEX.store(index, Ordering::Relaxed);
            ALLOCATED_FRAMES.fetch_add(1, Ordering::Relaxed);

            return Some(frame);
        }

        index += 1;
        next = 0;
    }

    None
}

/// Returns the number of allocates frames
pub fn allocated_frames() -> u64 {
    ALLOCATED_FRAMES.load(Ordering::Relaxed)
}

/// Returns the total number of usable frames discovered during initialization
pub fn total_usable_frames() -> u64 {
    TOTAL_USABLE_FRAMES.load(Ordering::Relaxed)
}

/// Logs the current frame allocator state
pub fn log_state() {
    crate::kprintln!(
        "[NX][FRAME] total_usable_bytes={:#018x} total_usable_frames={} allocated_frames={}",
        TOTAL_USABLE_BYTES.load(Ordering::Relaxed),
        total_usable_frames(),
        allocated_frames()
    );

    crate::kprintln!(
        "[NX][FRAME] current_entry={} next_frame={:#018x}",
        CURRENT_ENTRY_INDEX.load(Ordering::Relaxed),
        NEXT_FRAME.load(Ordering::Relaxed)
    );
}

/// Allocates a few frames and logs the result
/// 
/// The frames are intentionally not freed because this early allocator does not
/// support deallocation yet.
pub fn debug_smoke_test() {
    crate::kprintln!("[NX][FRAME] smoke test begin");

    for slot in 0..3 {
        match allocate_frame() {
            Some(frame) => {
                crate::kprintln!(
                    "[NX][FRAME] smoke frame {} physical={:#018x}",
                    slot,
                    frame.start
                );
            }
            None => {
                crate::kprintln!("[NX][FRAME] smoke frame {} allocation failed", slot);
            }
        }
    }

    log_state();

    crate::kprintln!("[NX][FRAME] smoke test complete");
}

/// Aligns an address upward to a 4 KiB boundary
fn align_up_4k(address: u64) -> u64 {
    (address + FRAME_SIZE - 1) & !(FRAME_SIZE - 1)
}

/// Aligns an address downward to a 4 KiB boundary
fn align_down_4k(address: u64) -> u64 {
    address & !(FRAME_SIZE - 1)
}

/// Returns whether the early physical frame allocator has been initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}