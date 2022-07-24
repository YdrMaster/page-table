//! x

#![no_std]
#![deny(warnings, unsafe_code, unstable_features, missing_docs)]

mod flags;
mod page_table;
mod pte;

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

pub use arch::*;
pub use flags::MmuFlags;
pub use page_table::{PageTable, PtQuery};
pub use pte::Pte;

/// 最小的分页大小。
///
/// 似乎任何架构都是或支持 4kiB 分页，而且对齐参数必须是字面量，所以此处直接做成常量。
pub const PAGE_SIZE: usize = 4096;

/// 页内偏移的位数
pub const OFFSET_BITS: usize = PAGE_SIZE.trailing_zeros() as _;

/// 每级页表容纳的页数
const ENTRIES_PER_TABLE: usize = PAGE_SIZE / core::mem::size_of::<usize>();

/// 每级页表的序号位数
pub const PT_LEVEL_BITS: usize = ENTRIES_PER_TABLE.trailing_zeros() as _;

/// 序号遮罩
const PT_LEVEL_MASK: usize = (1 << PT_LEVEL_BITS) - 1;

/// 物理地址。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct PPN(pub usize);

/// 虚拟地址。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct VAddr(usize);

impl VAddr {
    /// 将一个地址值转换为虚拟地址意味着允许虚存方案根据实际情况裁剪地址值。
    /// 超过虚址范围的地址会被裁剪。
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// 获得虚地址的地址值。
    #[inline]
    pub const fn value(self) -> usize {
        self.0
    }
}

impl From<usize> for VAddr {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl<T> From<*const T> for VAddr {
    #[inline]
    fn from(value: *const T) -> Self {
        Self(value as _)
    }
}

impl<T> From<&T> for VAddr {
    #[inline]
    fn from(value: &T) -> Self {
        Self(value as *const _ as _)
    }
}

/// 分页元数据。
pub trait MmuMeta: Copy {
    /// 物理地址位数，用于计算物理页号形式。
    const P_ADDR_BITS: usize;

    /// 虚拟页号位数，用于裁剪或扩展正确的虚址。
    const V_ADDR_BITS: usize;

    /// 物理地址在 PTE 中的位置。
    const PPN_BASE: usize;

    /// 从 PTE 中遮罩出 PPN，用于修改 PPN。
    ///
    /// ## NOTE
    ///
    /// 永远不必设置这个常量，因为它是自动计算的。
    const PPN_MASK: usize = ppn_mask(Self::PPN_BASE, Self::P_ADDR_BITS - OFFSET_BITS);

    /// 通过虚址位数计算页表最大级别。
    const MAX_LEVEL: usize = calculate_max_level(Self::V_ADDR_BITS);

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

    /// 如果页表项指向物理页，则返回 `true`。
    ///
    /// ## NOTE
    ///
    /// 为了零开销抽象，这个方法的实现可能不会判断 PTE 是否 valid。
    fn is_leaf(value: usize) -> bool;

    /// 判断页表项指向的是一个大于 0 级（4 kiB）的物理页。
    ///
    /// ## NOTE
    ///
    /// 为了零开销抽象，这个方法的实现可能不会判断 PTE 是否 valid。
    #[inline]
    fn is_huge(value: usize, level: usize) -> bool {
        level != 0 && Self::is_leaf(value)
    }

    /// 判断页表项是否 valid。
    #[inline]
    fn is_valid(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_D) != 0
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
    fn ppn(value: usize) -> PPN {
        PPN((value & Self::PPN_MASK) >> Self::PPN_BASE)
    }

    /// 设置页表项的 ppn。
    #[inline]
    fn set_ppn(value: &mut usize, ppn: PPN) {
        *value |= (ppn.0 << Self::PPN_BASE) & Self::PPN_MASK;
    }

    /// 清除页表项中的 ppn。
    #[inline]
    fn clear_ppn(value: &mut usize) {
        *value &= !Self::PPN_MASK;
    }
}

#[inline]
const fn calculate_max_level(v_addr_bits: usize) -> usize {
    (v_addr_bits - OFFSET_BITS + PT_LEVEL_BITS - 1) / PT_LEVEL_BITS - 1
}

#[inline]
const fn ppn_mask(base: usize, len: usize) -> usize {
    let m0: usize = !((1 << base) - 1);
    let m1: usize = (1 << (base + len)) - 1;
    m0 & m1
}

use static_assertions::const_assert_eq;

const_assert_eq!(PAGE_SIZE, 4096);
const_assert_eq!(OFFSET_BITS, 12);

cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        const_assert_eq!(PT_LEVEL_BITS, 10);
        const_assert_eq!(ENTRIES_PER_TABLE, 1024);
        const_assert_eq!(calculate_max_level(32), 1);
    } else if #[cfg(target_pointer_width = "64")] {
        const_assert_eq!(PT_LEVEL_BITS, 9);
        const_assert_eq!(ENTRIES_PER_TABLE, 512);
        const_assert_eq!(calculate_max_level(39), 2);
        const_assert_eq!(calculate_max_level(48), 3);
        const_assert_eq!(calculate_max_level(57), 4);
    } else {
        compile_error!("Unsupported architecture");
    }
}
