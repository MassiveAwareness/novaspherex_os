//! Early physical-to-virtual address helpers
//! 
//! This module uses Limine's Higher-Half Direct Map offset to construct virtual
//! addresses for physical addresses.
//! 
//! Important: this helper only computes the direct-map virtual address. It does
//! not prove that the physical region is actually mapped by Limine. Device MMIO
//! ranges such as the Local APIC may require explicit page table mappings later.

#![allow(dead_code)]

/// Initializes address diagnostics for early boot
pub fn init() {
    crate::limine::log_hhdm();
}

/// Returns the current HHDM offset, if Limine provided one
pub fn hhdm_offset() -> Option<u64> {
    crate::limine::hhdm_offset()
}

/// Converts a physical address to a higher-half direct map virtual address
/// 
/// This is a pure address calculation. The caller must still know whether the
/// physical region is actually mapped in the current page tables.
pub fn physical_to_virtual_address(physical: u64) -> Option<u64> {
    Some(hhdm_offset()?.wrapping_add(physical))
}

/// Converts a physical address to a mutable pointer in the HHDM
/// 
/// # Safety
/// The returned pointer is only safe to dereference if the physical address is
/// actually mapped and if the target memory/device permits the requested access.
pub unsafe fn physical_to_virtual_mut<T>(physical: u64) -> Option<*const T> {
    Some(physical_to_virtual_address(physical)? as *const T)
}

/// Converts a physical address to a const pointer in the HHDM
/// 
/// # Safety
/// The returned pointer is only safe to dereference if the physical address is
/// actually mapped and readable.
pub unsafe fn physical_to_virtual<T>(physical: u64) -> Option<*const T> {
    Some(physical_to_virtual_address(physical)? as *const T)
}