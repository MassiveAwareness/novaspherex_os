#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub fn init() {
    x86_64::init();
}

pub fn test_breakpoint() {
    x86_64::test_breakpoint();
}

pub fn test_page_fault() {
    x86_64::test_page_fault();
}