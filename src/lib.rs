//! # RustyHermit's entry API.

#![no_std]
#![cfg_attr(feature = "kernel", feature(const_ptr_offset_from))]
#![cfg_attr(feature = "kernel", feature(const_refs_to_cell))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![forbid(unsafe_code)]

#[cfg(feature = "loader")]
mod loader;

#[cfg(feature = "kernel")]
mod kernel;

#[cfg(feature = "kernel")]
pub use kernel::_Note;

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub type Entry = unsafe extern "C" fn(raw_boot_info: &'static RawBootInfo) -> !;

mod consts {
    /// Note type for specifying the hermit entry version.
    ///
    /// The note name for this is `HERMIT`.
    ///
    /// The `desc` field will be 1 word, which specifies the hermit entry version.
    pub const NT_HERMIT_ENTRY_VERSION: u32 = 0x5a00;

    pub const HERMIT_ENTRY_VERSION: u8 = 1;
}

#[cfg(feature = "loader")]
pub use consts::NT_HERMIT_ENTRY_VERSION;

#[cfg(feature = "loader")]
pub use consts::HERMIT_ENTRY_VERSION;

#[cfg(target_arch = "x86_64")]
type SerialPortBase = u16;
#[cfg(target_arch = "aarch64")]
type SerialPortBase = u32;

#[derive(Debug)]
pub struct BootInfo {
    /// Lowest physical memory address.
    #[cfg(target_arch = "aarch64")]
    pub ram_start: u64,

    /// Highest physical memory address.
    pub limit: u64,

    /// Start address of the loaded kernel image.
    pub base: u64,

    /// Size of the loaded kernel image in bytes.
    pub image_size: u64,

    /// Kernel image TLS information.
    pub tls_info: TlsInfo,

    /// Serial port base address.
    pub uartport: SerialPortBase,

    /// Discriminant determines if running on uhyve.
    pub uhyve: u8,

    /// UHYVE ONLY: Boot time as Unix timestamp in microseconds.
    pub boot_gtod: u64,

    /// UHYVE ONLY: CPU frequency in MHz.
    pub cpu_freq: u16,

    /// UHYVE ONLY: Total number of CPUs available.
    pub possible_cpus: u32,

    /// MULTIBOOT ONLY: Command line pointer.
    pub cmdline: u64,

    /// MULTIBOOT ONLY: Command line length.
    pub cmdsize: u64,

    /// MULTIBOOT ONLY: Multiboot boot information address.
    #[cfg(target_arch = "x86_64")]
    pub mb_info: u64,
}

#[derive(Debug)]
pub struct TlsInfo {
    pub start: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

#[derive(Debug)]
#[repr(C)]
pub struct RawBootInfo {
    /// Magic number (legacy)
    ///
    /// Used for identifying a `RawBootInfo` struct.
    magic_number: u32,

    /// Boot info version (legacy)
    ///
    /// Used to agree on the layout of `RawBootInfo`.
    /// Not necessary since the introduction of the entry version note.
    version: u32,

    base: u64,
    #[cfg(target_arch = "aarch64")]
    ram_start: u64,
    limit: u64,
    image_size: u64,
    tls_start: u64,
    tls_filesz: u64,
    tls_memsz: u64,
    #[cfg(target_arch = "aarch64")]
    tls_align: u64,

    /// The current stack address.
    current_stack_address: AtomicU64,

    /// The current percore address (legacy).
    ///
    /// libhermit-rs now uses an internal statically allocated variable.
    current_percore_address: u64,

    /// Virtual host address (legacy)
    ///
    /// Used by HermitCore for sharing a memory pool with uhyve at the same host and guest virtual address.
    ///
    /// <https://github.com/hermitcore/libhermit/commit/9a28225424519cd6ab2b42fb5a2997455ba03242>
    host_logical_addr: u64,

    boot_gtod: u64,
    #[cfg(target_arch = "x86_64")]
    mb_info: u64,
    cmdline: u64,
    cmdsize: u64,
    cpu_freq: u32,

    /// CPU ID of the boot processor (legacy)
    ///
    /// Used by HermitCore to identify the processor core that is the boot processor.
    /// libhermit-rs defaults to 0.
    boot_processor: u32,

    /// Number of initialized CPUs.
    ///
    /// Synchronizes vCPU startup with uhyve.
    cpu_online: AtomicU32,

    possible_cpus: u32,

    /// CPU ID of the currently booting processor (legacy)
    ///
    /// Used by HermitCore to identify the processor core that is currently booting.
    /// libhermit-rs deduces this from `cpu_online`.
    current_boot_id: u32,

    uartport: SerialPortBase,

    /// Single Kernel (legacy)
    ///
    /// This bool was used to determine whether HermitCore is the only kernel on the machine
    /// or if it is running in multikernel mode side by side with Linux.
    single_kernel: u8,

    uhyve: u8,

    /// Uhyve IP Address (legacy)
    ///
    /// Was used by lwIP once.
    hcip: [u8; 4],

    /// Uhyve Gateway Address (legacy)
    ///
    /// Was used by lwIP once.
    hcgateway: [u8; 4],

    /// Uhyve Network Mask (legacy)
    ///
    /// Was used by lwIP once.
    hcmask: [u8; 4],

    #[cfg(target_arch = "x86_64")]
    tls_align: u64,
}

impl RawBootInfo {
    pub fn store_current_stack_address(&self, current_stack_address: u64) {
        self.current_stack_address
            .store(current_stack_address, Ordering::Relaxed);
    }

    pub fn load_cpu_online(&self) -> u32 {
        self.cpu_online.load(Ordering::Acquire)
    }
}
