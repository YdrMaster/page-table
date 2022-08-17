//! x

#![no_std]
#![deny(warnings, unstable_features, missing_docs)]

mod addr;
mod flags;
mod page_table;
mod pte;
pub mod walker;

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        #[path = "arch/riscv.rs"]
        mod arch;
    } else if #[cfg(target_arch = "aarch64")] {
        #[path = "arch/arm.rs"]
        mod arch;
    } else if #[cfg(target_arch = "x86_64")] {
        #[path = "arch/x86.rs"]
        mod arch;
    } else {
        compile_error!("Unsupported architecture");
    }
}

pub use addr::{VAddr, PPN, VPN};
pub use arch::*;
pub use flags::VmFlags;
pub use page_table::{PageTable, PtQuery};
pub use pte::Pte;

/// 地址转换单元元数据。
pub trait MmuMeta {
    /// 物理地址位数，用于计算物理页号形式。
    const P_ADDR_BITS: usize;

    /// 各级页内虚地址位数位数。
    const LEVEL_BITS: &'static [usize];

    /// 页表项有效。
    const FLAG_POS_V: usize;

    /// 页可读。
    const FLAG_POS_R: usize;

    /// 页可写。
    const FLAG_POS_W: usize;

    /// 页可执行。
    const FLAG_POS_X: usize;

    /// 页可用户态访问。
    const FLAG_POS_U: usize;

    /// 全局页。
    const FLAG_POS_G: usize;

    /// 页已访问。
    const FLAG_POS_A: usize;

    /// 页已写。
    const FLAG_POS_D: usize;

    /// 物理页号在 PTE 中的位置。
    const PPN_POS: usize;

    /// 如果页表项指向物理页，则返回 `true`。
    ///
    /// # NOTE
    ///
    /// 为了分散开销，这个方法的实现不会判断 PTE 是否 valid。
    fn is_leaf(value: usize) -> bool;
}

/// 页式虚存元数据。
pub trait VmMeta: 'static + MmuMeta + Copy + Ord + core::hash::Hash + core::fmt::Debug {
    /// 虚拟页号位数，用于裁剪或扩展正确的虚址。
    const V_ADDR_BITS: usize = const_sum(0, Self::LEVEL_BITS);

    /// 页内偏移的位数
    const PAGE_BITS: usize = Self::LEVEL_BITS[0];

    /// 页表最大级别。
    const MAX_LEVEL: usize = Self::LEVEL_BITS.len() - 1;

    /// `level` 级页表容纳的页数。
    #[inline]
    fn pages_in(level: usize) -> usize {
        1 << Self::LEVEL_BITS[level]
    }

    /// 判断页表项指向的是一个大于 0 级（4 kiB）的物理页。
    ///
    /// # NOTE
    ///
    /// 为了零开销抽象，这个方法的实现可能不会判断 PTE 是否 valid。
    #[inline]
    fn is_huge(value: usize, level: usize) -> bool {
        level != 0 && Self::is_leaf(value)
    }

    /// 判断页表项是否 valid。
    #[inline]
    fn is_valid(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_V) != 0
    }

    /// 判断页表项是否可读。
    #[inline]
    fn is_readable(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_R) != 0
    }

    /// 判断页表项是否可写。
    #[inline]
    fn is_writable(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_W) != 0
    }

    /// 判断页表项是否可执行。
    #[inline]
    fn is_executable(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_X) != 0
    }

    /// 判断页表项是否用于用户态。
    #[inline]
    fn is_user(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_U) != 0
    }

    /// 判断页表项是否全局的。
    #[inline]
    fn is_global(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_G) != 0
    }

    /// 判断页表项是否访问过。
    #[inline]
    fn is_accessed(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_A) != 0
    }

    /// 判断页表项是否被修改过。
    #[inline]
    fn is_dirty(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_D) != 0
    }

    /// 从 PTE 中获得 PPN。
    #[inline]
    fn ppn(value: usize) -> PPN<Self> {
        PPN::new((value & ppn_mask::<Self>()) >> Self::PPN_POS)
    }

    /// 设置页表项的 ppn。
    #[inline]
    fn set_ppn(value: &mut usize, ppn: PPN<Self>) {
        *value |= (ppn.val() << Self::PPN_POS) & ppn_mask::<Self>();
    }

    /// 清除页表项中的 ppn。
    #[inline]
    fn clear_ppn(value: &mut usize) {
        *value &= !ppn_mask::<Self>();
    }
}

/// 自动实现。
impl<T: 'static + MmuMeta + Copy + Ord + core::hash::Hash + core::fmt::Debug> VmMeta for T {}

#[inline]
const fn mask(bits: usize) -> usize {
    (1 << bits) - 1
}

#[inline]
const fn ppn_mask<Meta: MmuMeta>() -> usize {
    let m0: usize = !mask(Meta::PPN_POS);
    let m1: usize = mask(Meta::PPN_POS + Meta::P_ADDR_BITS - Meta::LEVEL_BITS[0]);
    m0 & m1
}

#[inline]
const fn const_sum(val: usize, bits: &[usize]) -> usize {
    match bits {
        [] => val,
        [n, tail @ ..] => const_sum(val + *n, tail),
    }
}
