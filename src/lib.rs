#![no_std]

mod arch;

pub use arch::*;

use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

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
pub struct VAddr(pub usize);

/// 页表项。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct Pte<Meta: MmuMeta>(pub usize, PhantomData<Meta>);

/// MMU 属性。
///
/// MMU 属性一定完全包含在页表项中，所以独立的 MMU 属性实现为一个无法获取地址的页表项。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MmuFlags<Meta: MmuMeta>(pub usize, PhantomData<Meta>);

/// 页表。
#[repr(C, align(4096))]
pub struct PageTable<Meta: MmuMeta>([Pte<Meta>; ENTRIES_PER_TABLE]);

/// 查询结果。
#[derive(Clone, Copy, Debug)]
pub enum QueryPte<Meta: MmuMeta> {
    /// 虚存对应的页表项不存在。
    Invalid,
    /// 下一级页表的物理地址。
    SubTable(PPN),
    /// 页表项。
    Leaf(Pte<Meta>),
}

/// 分页元数据。
pub trait MmuMeta: Copy {
    const ADDR_MASK: usize;

    const MAX_LEVEL: usize;
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

impl<Meta: MmuMeta> Pte<Meta> {
    pub const ZERO: Self = Self(0, PhantomData);

    #[inline]
    pub fn ppn(self) -> PPN {
        Meta::ppn(self.0)
    }

    #[inline]
    pub fn is_leaf(self) -> bool {
        Meta::is_leaf(self.0)
    }

    #[inline]
    pub fn is_huge(self, level: usize) -> bool {
        Meta::is_huge(self.0, level)
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        Meta::is_valid(self.0)
    }

    #[inline]
    pub fn is_readable(self) -> bool {
        Meta::is_readable(self.0)
    }

    #[inline]
    pub fn is_writable(self) -> bool {
        Meta::is_writable(self.0)
    }

    #[inline]
    pub fn is_executable(self) -> bool {
        Meta::is_executable(self.0)
    }

    #[inline]
    pub fn is_user(self) -> bool {
        Meta::is_user(self.0)
    }

    #[inline]
    pub fn is_global(self) -> bool {
        Meta::is_global(self.0)
    }

    #[inline]
    pub fn is_accessed(self) -> bool {
        Meta::is_accessed(self.0)
    }

    #[inline]
    pub fn is_dirty(self) -> bool {
        Meta::is_dirty(self.0)
    }

    #[inline]
    pub fn set_ppn(&mut self, ppn: PPN) {
        Meta::clear_ppn(&mut self.0);
        Meta::set_ppn(&mut self.0, ppn);
    }

    #[inline]
    pub fn flags(mut self) -> MmuFlags<Meta> {
        Meta::clear_ppn(&mut self.0);
        MmuFlags(self.0, PhantomData)
    }
}

impl<Meta: MmuMeta> MmuFlags<Meta> {
    pub const ZERO: Self = Self(0, PhantomData);

    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value, PhantomData)
    }

    #[inline]
    pub fn is_leaf(self) -> bool {
        Meta::is_leaf(self.0)
    }

    #[inline]
    pub fn is_huge(self, level: usize) -> bool {
        Meta::is_huge(self.0, level)
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        Meta::is_valid(self.0)
    }

    #[inline]
    pub fn is_readable(self) -> bool {
        Meta::is_readable(self.0)
    }

    #[inline]
    pub fn is_writable(self) -> bool {
        Meta::is_writable(self.0)
    }

    #[inline]
    pub fn is_executable(self) -> bool {
        Meta::is_executable(self.0)
    }

    #[inline]
    pub fn is_user(self) -> bool {
        Meta::is_user(self.0)
    }

    #[inline]
    pub fn is_global(self) -> bool {
        Meta::is_global(self.0)
    }

    #[inline]
    pub fn is_accessed(self) -> bool {
        Meta::is_accessed(self.0)
    }

    #[inline]
    pub fn is_dirty(self) -> bool {
        Meta::is_dirty(self.0)
    }

    #[inline]
    pub fn build_pte(mut self, ppn: PPN) -> Pte<Meta> {
        Meta::set_ppn(&mut self.0, ppn);
        Pte(self.0, PhantomData)
    }
}

impl<Meta: MmuMeta> PageTable<Meta> {
    pub const ZERO: Self = Self([Pte::ZERO; ENTRIES_PER_TABLE]);

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        !self.0.iter().any(|pte| pte.is_valid())
    }

    pub fn set_entry(&mut self, vaddr: VAddr, entry: Pte<Meta>, level: usize) -> bool {
        if level >= Meta::MAX_LEVEL {
            return false;
        }

        let vpn = (vaddr.0 >> (OFFSET_BITS + level * PT_LEVEL_BITS)) & PT_LEVEL_MASK;
        self.0[vpn] = entry;

        true
    }

    pub fn query(&self, addr: VAddr, level: u8) -> QueryPte<Meta> {
        let mut idx = addr.0 >> OFFSET_BITS;
        for _ in 0..level {
            idx >>= PT_LEVEL_BITS;
        }
        idx &= (1 << PT_LEVEL_BITS) - 1;
        let pte = self.0[idx];
        if !pte.is_valid() {
            QueryPte::Invalid
        } else if pte.is_leaf() {
            QueryPte::Leaf(pte)
        } else {
            QueryPte::SubTable(pte.ppn())
        }
    }
}

impl<Meta: MmuMeta> Index<usize> for PageTable<Meta> {
    type Output = Pte<Meta>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<Meta: MmuMeta> IndexMut<usize> for PageTable<Meta> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

use static_assertions::const_assert_eq;

const_assert_eq!(PAGE_SIZE, 4096);
const_assert_eq!(OFFSET_BITS, 12);

cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        const_assert_eq!(ENTRIES_PER_TABLE, 1024);
    } else if #[cfg(target_pointer_width = "64")] {
        const_assert_eq!(ENTRIES_PER_TABLE, 512);
    } else {
        compile_error!("Unsupported architecture");
    }
}
