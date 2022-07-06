#![no_std]

mod arch;
mod flags;
mod page_table;
mod pte;

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
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

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
    const ADDR_MASK: usize;

    const V_ADDR_BITS: usize;
    const MAX_LEVEL: usize = calculate_max_level(Self::V_ADDR_BITS);
    const FLAG_POS_V: usize;
    const FLAG_POS_R: usize;
    const FLAG_POS_W: usize;
    const FLAG_POS_X: usize;
    const FLAG_POS_U: usize;
    const FLAG_POS_G: usize;
    const FLAG_POS_A: usize;
    const FLAG_POS_D: usize;

    fn is_leaf(value: usize) -> bool;

    #[inline]
    fn is_huge(value: usize, level: usize) -> bool {
        level < Self::MAX_LEVEL && Self::is_leaf(value)
    }

    fn ppn(value: usize) -> PPN;

    #[inline]
    fn is_valid(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_D) != 0
    }

    #[inline]
    fn is_readable(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_R) != 0
    }

    #[inline]
    fn is_writable(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_W) != 0
    }

    #[inline]
    fn is_executable(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_X) != 0
    }

    #[inline]
    fn is_user(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_U) != 0
    }

    #[inline]
    fn is_global(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_G) != 0
    }

    #[inline]
    fn is_accessed(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_A) != 0
    }

    #[inline]
    fn is_dirty(value: usize) -> bool {
        value & (1 << Self::FLAG_POS_D) != 0
    }

    fn set_ppn(value: &mut usize, ppn: PPN);

    fn clear_ppn(value: &mut usize);
}

#[inline]
const fn calculate_max_level(v_addr_bits: usize) -> usize {
    (v_addr_bits - OFFSET_BITS + PT_LEVEL_BITS - 1) / PT_LEVEL_BITS - 1
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
