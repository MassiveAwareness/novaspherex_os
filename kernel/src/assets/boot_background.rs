//! Retro16 boot background assets
//! 
//! Source PNG files are stored in `assets/images`
//! `tools/prepare-assets.ps1` converts them into raw RGBX pixel buffers that
//! can be embedded directly into the kernel with `include_bytes!` macro.

#![allow(dead_code)]

/// Logical boot background width
pub const BOOT_BACKGROUND_WIDTH: usize = 640;

/// Logical boot background height
pub const BOOT_BACKGROUND_HEIGHT: usize = 360;

/// Bytes per pixel in the generated RGBX format
pub const BOOT_BACKGROUND_BYTES_PER_PIXEL: usize = 4;

/// Number of available boot backgrounds
pub const BOOT_BACKGROUND_COUNT: usize = 5;

/// Embedded boot background image
#[derive(Clone, Copy)]
pub struct BootBackground {
    /// Human-facing asset id, matching `bg_<id>.png`
    pub id: u8,

    /// Width in pixels
    pub width: usize,

    /// Height in pixels
    pub height: usize,

    /// Raw RGBX pixel data
    pub data: &'static [u8]
}

const BG_1: &[u8] = include_bytes!("generated\\bg_1.rgbx");
const BG_2: &[u8] = include_bytes!("generated\\bg_2.rgbx");
const BG_3: &[u8] = include_bytes!("generated\\bg_3.rgbx");
const BG_4: &[u8] = include_bytes!("generated\\bg_4.rgbx");
const BG_5: &[u8] = include_bytes!("generated\\bg_5.rgbx");

const BACKGROUNDS: [BootBackground; BOOT_BACKGROUND_COUNT] = [
    BootBackground {
        id: 1,
        width: BOOT_BACKGROUND_WIDTH,
        height: BOOT_BACKGROUND_HEIGHT,
        data: BG_1
    },
    BootBackground {
        id: 2,
        width: BOOT_BACKGROUND_WIDTH,
        height: BOOT_BACKGROUND_HEIGHT,
        data: BG_2
    },
    BootBackground {
        id: 3,
        width: BOOT_BACKGROUND_WIDTH,
        height: BOOT_BACKGROUND_HEIGHT,
        data: BG_3
    },
    BootBackground {
        id: 4,
        width: BOOT_BACKGROUND_WIDTH,
        height: BOOT_BACKGROUND_HEIGHT,
        data: BG_4
    },
    BootBackground {
        id: 5,
        width: BOOT_BACKGROUND_WIDTH,
        height: BOOT_BACKGROUND_HEIGHT,
        data: BG_5
    }
];

/// Selects one boot background using a non-cryptographic seed
/// The returned id is effective in the inclusive range `1..=5`.
pub fn select(seed: u64) -> BootBackground {
    let index = (seed as usize) % BOOT_BACKGROUND_COUNT;
    BACKGROUNDS[index]
}