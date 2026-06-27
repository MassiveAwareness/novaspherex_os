#![allow(dead_code)]

use core::ptr;

const COMMON_MAGIC: [u64; 2] = [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b];

const BOOTLOADER_INFO_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0xf55038d8e2a1202f,
    0x279426fcf5f59740,
];

const FRAMEBUFFER_REQUEST_ID: [u64; 4] = [
    COMMON_MAGIC[0],
    COMMON_MAGIC[1],
    0x9d5827dcd881dd75,
    0xa3148604f6fab11b,
];

#[used]
#[link_section = ".limine_requests_start"]
static LIMINE_REQUESTS_START_MARKER: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

#[used]
#[link_section = ".limine_requests"]
static mut LIMINE_BASE_REVISION: [u64; 3] = [
    0xf9562b2d5c95a6c8,
    0x6a7b384944536bdc,
    6,
];

#[used]
#[link_section = ".limine_requests"]
static mut BOOTLOADER_INFO_REQUEST: BootloaderInfoRequest = BootloaderInfoRequest {
    id: BOOTLOADER_INFO_REQUEST_ID,
    revision: 0,
    response: ptr::null_mut(),
};

#[used]
#[link_section = ".limine_requests"]
static mut FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest {
    id: FRAMEBUFFER_REQUEST_ID,
    revision: 0,
    response: ptr::null_mut(),
};

#[used]
#[link_section = ".limine_requests_end"]
static LIMINE_REQUESTS_END_MARKER: [u64; 2] = [
    0xadc0e0531bb10d03,
    0x9572709f31764c62,
];

#[repr(C)]
pub struct BootloaderInfoRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut BootloaderInfoResponse,
}

#[repr(C)]
pub struct BootloaderInfoResponse {
    revision: u64,
    pub name: *mut i8,
    pub version: *mut i8,
}

#[repr(C)]
pub struct FramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut FramebufferResponse,
}

#[repr(C)]
pub struct FramebufferResponse {
    revision: u64,
    framebuffer_count: u64,
    framebuffers: *mut *mut Framebuffer,
}

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

pub fn base_revision_supported() -> bool {
    unsafe {
        let base = ptr::addr_of!(LIMINE_BASE_REVISION).cast::<u64>();
        ptr::read_volatile(base.add(2)) == 0
    }
}

pub unsafe fn bootloader_info() -> Option<&'static BootloaderInfoResponse> {
    let request = ptr::addr_of!(BOOTLOADER_INFO_REQUEST);
    (*request).response.as_ref()
}

pub unsafe fn draw_boot_banner() {
    let request = ptr::addr_of!(FRAMEBUFFER_REQUEST);
    let Some(response) = (*request).response.as_ref() else {
        return;
    };

    if response.framebuffer_count == 0 || response.framebuffers.is_null() {
        return;
    }

    let framebuffer = *response.framebuffers;
    if framebuffer.is_null() {
        return;
    }

    let fb = &*framebuffer;
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
            write_pixel(fb.address.add(offset as usize), bytes_per_pixel, pixel);
        }
    }
}

fn min_u64(a: u64, b: u64) -> u64 {
    if a < b { a } else { b }
}

fn pack_pixel(fb: &Framebuffer, r: u8, g: u8, b: u8) -> u32 {
    let r = scale_component(r, fb.red_mask_size) << fb.red_mask_shift;
    let g = scale_component(g, fb.green_mask_size) << fb.green_mask_shift;
    let b = scale_component(b, fb.blue_mask_size) << fb.blue_mask_shift;
    r | g | b
}

fn scale_component(value: u8, mask_size: u8) -> u32 {
    if mask_size == 0 {
        0
    } else if mask_size >= 8 {
        value as u32
    } else {
        ((value as u32) >> (8 - mask_size)) & ((1u32 << mask_size) - 1)
    }
}

unsafe fn write_pixel(dst: *mut u8, bytes_per_pixel: u64, pixel: u32) {
    match bytes_per_pixel {
        4 => ptr::write_volatile(dst as *mut u32, pixel),
        3 => {
            ptr::write_volatile(dst, pixel as u8);
            ptr::write_volatile(dst.add(1), (pixel >> 8) as u8);
            ptr::write_volatile(dst.add(2), (pixel >> 16) as u8);
        }
        _ => {}
    }
}