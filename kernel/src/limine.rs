//! Minimal Limine boot protocol bindings
//! 
//! This module defines the Limine request structures used by the early kernel.
//! Limine discovers these statically-linked request objects through special
//! linker sections, fills in their response pointers during boot, and then
//! transfers control to `_start`.
//! 
//! We intentionally keep this module small and explicit isntead of hiding the
//! boot protocol behind a large abstraction. Early boot code benefits from
//! being easy to inspect.

#![allow(dead_code)]

use core::ptr;

/// Limine memory map type: usable RAM
pub const MEMORY_KIND_USABLE: u64 = 0;

/// Limine memory map type: reserved memory
pub const MEMORY_KIND_RESERVED: u64 = 1;

/// Limine memory map type: ACII reclaimable memory
pub const MEMORY_KIND_ACPI_RECLAIMABLE: u64 = 2;

/// Limine memory map type: ACPI non-volatile storage
pub const MEMORY_KIND_ACPI_NON_VOLATILE: u64 = 3;

/// Limine memory map type: bad memory
pub const MEMORY_KIND_BAD_MEMORY: u64 = 4;

/// Limine memory map type: bootloader reclaimable memory
pub const MEMORY_KIND_BOOTLOADER_RECLAIMABLE: u64 = 5;

/// Limine memory map type: kernel and modules memory
pub const MEMORY_KIND_KERNEL_AND_MODULES: u64 = 6;

/// Limine memory map type: framebuffer memory
pub const MEMORY_KIND_FRAMEBUFFER: u64 = 7;

/// Common Limine request magic prefix
/// 
/// Limine request IDs start with this common two-word magic value followed by
/// a request-specific identifier.
const COMMON_MAGIC: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];

/// Request ID for bootloader name/version information
const BOOTLOADER_INFO_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0xf55038d8e2a1202f,
    0x279426fcf5f59740,
];

/// Request ID for framebuffer information
const FRAMEBUFFER_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0x9d5827dcd881dd75,
    0xa3148604f6fab11b,
];

/// Request ID for Higher-Half Direct Map information
const HHDM_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0x48dcf1cb8ad2b852,
    0x63984e959a98244b
];

/// Request ID for the Limine memory map request
const MEMORY_MAP_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0x67cf_3d9d_378a_806f,
    0xe304_acdf_c50c_3c62
];

/// Start marker for Limine requests
/// 
/// The `#[used]` attribute prevents the compiler/linker from discarding this
/// symbol even though Rust code does not reference it directly.
#[used]
#[link_section = ".limine_requests_start"]
static LIMINE_REQUESTS_START_MARKER: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

/// Limine base revision request
/// 
/// Limine acknowledges this request by overwriting the thrid field with `0`.
/// We use this as a simple sanity check that the bootloader recognized our
/// request block.
#[used]
#[link_section = ".limine_requests"]
static mut LIMINE_BASE_REVISION: [u64; 3] = [
    0xf9562b2d5c95a6c8,
    0x6a7b384944536bdc,
    6,
];

/// Bootloader information request
/// 
/// Limine fills `response` with a pointer to a `BootloaderInfoResponse` if the
/// request is supported.
#[used]
#[link_section = ".limine_requests"]
static mut BOOTLOADER_INFO_REQUEST: BootloaderInfoRequest = BootloaderInfoRequest {
    id: BOOTLOADER_INFO_REQUEST_ID,
    revision: 0,
    response: ptr::null_mut(),
};

/// Framebuffer request
/// 
/// Limine fills `response` with framebuffer metadata if a framebuffer is
/// available.
#[used]
#[link_section = ".limine_requests"]
static mut FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest {
    id: FRAMEBUFFER_REQUEST_ID,
    revision: 0,
    response: ptr::null_mut(),
};

/// Higher-Half Direct Map request
/// 
/// Limine fills `response` with an HHDM offset if the request is supported.
#[used]
#[link_section = ".limine_requests"]
static mut HHDM_REQUEST: HhdmRequest = HhdmRequest {
    id: HHDM_REQUEST_ID,
    revision: 0,
    response: ptr::null_mut()
};

/// Memory map request
/// 
/// Limine fills this response with an array of physical memory regions. The
/// memory subsystem uses this to discover usable RAM and reserved regions.
#[used]
#[link_section = ".limine_requests"]
static mut MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest {
    id: MEMORY_MAP_REQUEST_ID,
    revision: 0,
    response: ptr::null_mut()
};

/// End marker for Limine requests
#[used]
#[link_section = ".limine_requests_end"]
static LIMINE_REQUESTS_END_MARKER: [u64; 2] = [
    0xadc0e0531bb10d03,
    0x9572709f31764c62,
];

/// Limine bootloader-info request structure
#[repr(C)]
pub struct BootloaderInfoRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut BootloaderInfoResponse,
}

/// Limine bootloader-info response
/// 
/// The `name` and `version` fields are null-terminated C strings owned by the
/// bootloader.
#[repr(C)]
pub struct BootloaderInfoResponse {
    revision: u64,
    pub name: *mut i8,
    pub version: *mut i8,
}

/// Limine framebuffer request structure
#[repr(C)]
pub struct FramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut FramebufferResponse,
}

/// Limine framebuffer response
/// 
/// `framebuffers` points to an array of framebuffer pointers.
#[repr(C)]
pub struct FramebufferResponse {
    revision: u64,
    framebuffer_count: u64,
    framebuffers: *mut *mut Framebuffer,
}

/// Limine video mode metadata
#[repr(C)]
pub struct VideoMode {
    pitch: u64,
    width: u64,
    height: u64,
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
}

/// Limine Framebuffer metadata
#[repr(C)]
pub struct Framebuffer {
    address: *mut u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    unused: [u8; 7],
    edid_size: u64,
    edid: *mut u8,
    mode_count: u64,
    modes: *mut *mut VideoMode,
}

/// Limine HHDM request structure
#[repr(C)]
pub struct HhdmRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut HhdmResponse
}

/// Limine HHDM response
///
/// `offset` is the virtual address offset used for the higher-half direct map.
/// A physical address `p` can be addressed as virtual address `offset + p`,
/// provided that the physical region is actually part of Limine's HHDM mapping.
#[repr(C)]
pub struct HhdmResponse {
    revision: u64,
    pub offset: u64
}

/// Limine memory map structure
pub struct MemoryMapRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut MemoryMapResponse
}

/// Limine memory map response
/// 
/// `entries` points to an array of pointers. Each pointer targets one memory
/// map entry.
#[repr(C)]
pub struct MemoryMapResponse {
    revision: u64,
    pub entry_count: u64,
    pub entries: *const *const MemoryMapEntry
}

/// Limine memory map entry
/// 
/// The `kind` field uses Limine memory map entry type values. We intentionally
/// store it as `u64` instead of a Rust enum so that unknown future values do not
/// create invalid enum discriminants.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub kind: u64
}

/// Returns whether Limine acknowledged the requested base revision
pub fn base_revision_supported() -> bool {
    // SAFETY: Limine may mutate this static before transferring control to the
    // kernel. After boot, we only read it using volatile access so the compiler
    // does not assume ordinary Rust ownership semantics for this boot protocol
    // memory.
    unsafe {
        let base = ptr::addr_of!(LIMINE_BASE_REVISION).cast::<u64>();
        ptr::read_volatile(base.add(2)) == 0
    }
}

/// Returns Limine bootloader information, if available
/// 
/// # Safety
/// The returned reference points to memory provided by the bootloader. The
/// called must only use it during early boot while Limine-provided structures
/// are still considered valid.
pub unsafe fn bootloader_info() -> Option<&'static BootloaderInfoResponse> {
    let request = ptr::addr_of!(BOOTLOADER_INFO_REQUEST);

    // SAFETY: The response pointer is written by Limine before entering the
    // kernel. We only conver a non-null response pointer into a shared
    // reference.
    unsafe {
        (*request).response.as_ref()
    }
}

/// Draws a simple early boot banner into the first framebuffer
/// 
/// This is intentionally primitive. It verifies that the framebuffer request is
/// usable and gives visual feedback before the real graphics subsystem exists.
/// 
/// # Safety
/// This writes directly to a framebuffer pointer provided by Limine. The caller
/// must ensure this is only used after Limine has initialized the framebuffer
/// response and before any other graphics owner exists.
pub unsafe fn draw_boot_banner() {
    let request = ptr::addr_of!(FRAMEBUFFER_REQUEST);

    // SAFETY: The response pointer is provided by Limine. A null response means
    // no framebuffer is available.
    let Some(response) = (unsafe { (*request).response.as_ref() }) else {
        return;
    };

    if response.framebuffer_count == 0 || response.framebuffers.is_null() {
        return;
    }

    // SAFETY: Limine reports at least one framebuffer and `framebuffers` is
    // non-null. We read the first framebuffer pointer.
    let framebuffer = unsafe { *response.framebuffers };
    if framebuffer.is_null() {
        return;
    }

    // SAFETY: The framebuffer pointer is provided by Limine and was checked for
    // null above.
    let fb = unsafe { &*framebuffer };
    if fb.address.is_null() || fb.bpp < 24 {
        return;
    }

    let width = min_u64(fb.width, 480);
    let height = min_u64(fb.height, 96);
    let bytes_per_pixel = (fb.bpp / 8) as u64;

    for y in 0..height {
        for x in 0..width {
            let (r, g, b) = if y < 24 {
                (0x20, 0x24, 0x2d)
            } else if y < 48 {
                (0x55, 0xb6, 0xff)
            } else if y < 72 {
                (0x9f, 0x7a, 0xff)
            } else {
                (0x12, 0x14, 0x18)
            };

            let pixel = pack_pixel(fb, r, g, b);
            let offset = y * fb.pitch + x * bytes_per_pixel;

            // SAFETY: The offset is bounded by the small boot banner region,
            // and the framebuffer pointer was provided by Limine. This is still
            // raw framebuffer access, so the caller-level safety contract
            // applies.
            unsafe {
                write_pixel(fb.address.add(offset as usize), bytes_per_pixel, pixel);
            }
        }
    }
}

/// Returns the smaller of two `u64` values
fn min_u64(a: u64, b: u64) -> u64 {
    if a < b { a } else { b }
}

/// Packs an RGB color according to the framebuffer's color masks
fn pack_pixel(fb: &Framebuffer, r: u8, g: u8, b: u8) -> u32 {
    let r = scale_component(r, fb.red_mask_size) << fb.red_mask_shift;
    let g = scale_component(g, fb.green_mask_size) << fb.green_mask_shift;
    let b = scale_component(b, fb.blue_mask_size) << fb.blue_mask_shift;
    r | g | b
}

/// Scales an 8-bit color component down to a framebuffer mask size
fn scale_component(value: u8, mask_size: u8) -> u32 {
    if mask_size == 0 {
        0
    } else if mask_size >= 8 {
        value as u32
    } else {
        ((value as u32) >> (8 - mask_size)) & ((1u32 << mask_size) - 1)
    }
}

/// Writes one packed pixel to the framebuffer
unsafe fn write_pixel(dst: *mut u8, bytes_per_pixel: u64, pixel: u32) {
    match bytes_per_pixel {
        4 => {
            // SAFETY: Caller guarantees that `dst` points to a valid
            // framebuffer pixel
            unsafe {
                ptr::write_volatile(dst as *mut u32, pixel)
            }
        }
        3 => {
            // SAFETY: Caller guarantees that `dst..dst+3` is a valid
            // framebuffer pixel
            unsafe {
                ptr::write_volatile(dst, pixel as u8);
                ptr::write_volatile(dst.add(1), (pixel >> 8) as u8);
                ptr::write_volatile(dst.add(2), (pixel >> 16) as u8);
            }
        }
        _ => {}
    }
}

/// Returns the Limine Higher-Half Direct Map offset, if available
pub fn hhdm_offset() -> Option<u64> {
    let request = ptr::addr_of!(HHDM_REQUEST);

    // SAFETY: Limine writes the response pointer before entering the kernel.
    // We only read the pointer and convert a non-null response to a shared
    // reference.
    let response = unsafe {
        (*request).response.as_ref()?
    };

    Some(response.offset)
}

/// Logs the HHDM offset for early boot diagnostics.
pub fn log_hhdm() {
    match hhdm_offset() {
        Some(offset) => {
            crate::kprintln!("[NX][HHDM] offset={:#018x}", offset);
        },
        None => {
            crate::kprintln!("[NX][HHDM] unavailable");
        }
    }
}

/// Draws a raw RGBX image scaled to the first framebuffer
/// 
/// The image format is:
/// - byte 0: red
/// - byte 1: green
/// - byte 2: blue
/// - byte 3: unused padding
/// 
/// The image os stretched to the framebuffer using nearest-neighbor sampling.
/// This preserves the hard-edged pixel-art character better than filtered scaling.
/// 
/// # Safety
/// This writes directly to a framebuffer pointer provided by Limine. The caller
/// must ensure that this runs during early boot before another graphics owner
/// exists.
pub unsafe fn draw_rgbx_image_scaled(image_width: usize, image_height: usize, data: &[u8]) {
    let request = ptr::addr_of!(FRAMEBUFFER_REQUEST);

    // SAFETY: The response pointer is provided by Limine. A null response means
    // no framebuffer is available.
    let Some(response) = (unsafe {
        (*request).response.as_ref()
    }) else {
        return;
    };
    if response.framebuffer_count == 0 || response.framebuffers.is_null() {
        return;
    }

    // SAFETY: Limine reports at least one framebuffer and `framebuffers` is
    // non-null. We read the first framebuffer pointer.
    let framebuffer = unsafe {
        *response.framebuffers
    };

    if framebuffer.is_null() {
        return;
    }

    // SAFETY: The framebuffer pointer is provided by Limine and checked for
    // null above.
    let fb = unsafe {
        &*framebuffer
    };

    if fb.address.is_null() || fb.bpp < 24 {
        return;
    }

    let required_len = image_width
        .saturating_mul(image_height)
        .saturating_mul(4);

    if data.len() < required_len {
        return;
    }

    let framebuffer_width = fb.width as usize;
    let framebuffer_height = fb.height as usize;
    let bytes_per_pixel = (fb.bpp / 8) as u64;

    for y in 0..framebuffer_height {
        let source_y = y.saturating_mul(image_height) / framebuffer_height;

        for x in 0..framebuffer_width {
            let source_x = x.saturating_mul(image_width) / framebuffer_width;
            let source_index = (source_y * image_width + source_x) * 4;

            let r = data[source_index];
            let g = data[source_index + 1];
            let b = data[source_index + 2];

            let pixel = pack_pixel(fb, r, g, b);
            let offset = y as u64 * fb.pitch + x as u64 * bytes_per_pixel;

            // SAFETY: The loop bounds target the reported framebuffer size, and
            // the framebuffer pointer is provided by Limine. This remains raw
            // framebuffer access, so the caller-level safety contract applies.
            unsafe {
                write_pixel(fb.address.add(offset as usize), bytes_per_pixel, pixel);
            }
        }
    }
}

/// Returns the Limine memory map response, if available
pub fn memory_map_response() -> Option<&'static MemoryMapResponse> {
    let request = ptr::addr_of!(MEMORY_MAP_REQUEST);

    // SAFETY: Limine writes the response pointer before entering the kernel.
    // We only read the pointer and convert a non-null response to a shared
    // reference. The kernel must not invalidate bootloader-provided memory while
    // using this response.
    unsafe {
        (*request).response.as_ref()
    }
}

/// Returns one memory entry by index
pub fn memory_map_entry(index: usize) -> Option<&'static MemoryMapEntry> {
    let response = memory_map_response()?;

    if index >= response.entry_count as usize || response.entries.is_null() {
        return None;
    }

    // SAFETY: Limine reports `entry_count` entries and `entries` points to an
    // array of entry pointers. Bounds and null checks are performed before
    // dereferencing.
    let entry_pointer = unsafe {
        *response.entries.add(index)
    };

    if entry_pointer.is_null() {
        return None;
    }

    // SAFETY: The entry pointer comes from Limine's memory map pointer array and
    // was checked for null above.
    Some(unsafe { &*entry_pointer })
}

/// Returns a readable name for a Limine memory map entry kind
pub fn memory_kind_name(kind: u64) -> &'static str {
    match kind {
        MEMORY_KIND_USABLE => "Usable",
        MEMORY_KIND_RESERVED => "Reserved",
        MEMORY_KIND_ACPI_RECLAIMABLE => "AcpiReclaimable",
        MEMORY_KIND_ACPI_NON_VOLATILE => "AcpiNonVolatile",
        MEMORY_KIND_BAD_MEMORY => "BadMemory",
        MEMORY_KIND_BOOTLOADER_RECLAIMABLE => "BootloaderReclaimable",
        MEMORY_KIND_KERNEL_AND_MODULES => "KernelAndModules",
        MEMORY_KIND_FRAMEBUFFER => "Framebuffer",
        _ => "Unknown"
    }
}