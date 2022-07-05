cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        #[path = "riscv.rs"]
        mod arch;
    } else if #[cfg(target_arch = "x86_64")] {
        #[path = "x86/mod.rs"]
        mod arch;
    } else if #[cfg(target_arch = "aarch64")] {
        #[path = "arm/mod.rs"]
        mod arch;
    } else {
        compile_error!("Unsupported architecture");
    }
}

pub use arch::*;
