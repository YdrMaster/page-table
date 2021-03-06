use crate::{
    MmuMeta, Pte, VAddr, ENTRIES_PER_TABLE, OFFSET_BITS, PPN, PT_LEVEL_BITS, PT_LEVEL_MASK,
};
use core::ops::{Index, IndexMut};

/// 页表。
#[repr(C, align(4096))]
pub struct PageTable<Meta: MmuMeta>([Pte<Meta>; ENTRIES_PER_TABLE]);

/// 查询结果。
#[derive(Clone, Copy, Debug)]
pub enum PtQuery<Meta: MmuMeta> {
    /// 虚存对应的页表项不存在。
    Invalid,
    /// 下一级页表的物理页号。
    SubTable(PPN),
    /// 页表项。
    Leaf(Pte<Meta>),
}

#[derive(Clone, Copy, Debug)]
pub enum EntryError {
    InvalidLevel,
    LeafMisaligned,
}

impl<Meta: MmuMeta> PageTable<Meta> {
    /// 空白页表。
    pub const ZERO: Self = Self([Pte::ZERO; ENTRIES_PER_TABLE]);

    /// 页表长度。
    ///
    /// 总长度，而非有效项的数量。这是一个常量。
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// 如果页表中至少一个项有效，返回 `true`。
    #[inline]
    pub fn is_empty(&self) -> bool {
        !self.0.iter().any(|pte| pte.is_valid())
    }

    /// 获取指向第一个页表项的指针。
    #[inline]
    pub fn as_ptr(&self) -> *const Pte<Meta> {
        self.0.as_ptr()
    }

    /// 将此页表视为一个 `level` 级页表，设置一个页表项。
    ///
    /// 页表项保证查找虚地址 `vaddr` 时能找到页表项 `entry` 指向的物理页（可以是页或子页表）。
    ///
    /// # Errors
    ///
    /// - 如果 `level` 大于当前方案下最大的页表级别，产生 [`InvalidLevel`](EntryError::InvalidLevel)；
    /// - 如果 `entry` 在 `level` 级页表中表示一个巨页但物理页号未对齐，产生 [`LeafMisaligned`](EntryError::LeafMisaligned)；
    pub fn set_entry(
        &mut self,
        vaddr: VAddr,
        entry: Pte<Meta>,
        level: usize,
    ) -> Result<(), EntryError> {
        if level > Meta::MAX_LEVEL {
            Err(EntryError::InvalidLevel)?;
        }
        let page_align = level * PT_LEVEL_BITS;
        if entry.is_huge(level) && (entry.ppn().0.trailing_zeros() as usize) < page_align {
            Err(EntryError::LeafMisaligned)?;
        }
        let vpn = (vaddr.0 >> (OFFSET_BITS + page_align)) & PT_LEVEL_MASK;
        self.0[vpn] = entry;
        Ok(())
    }

    /// 查询页表一次。
    pub fn query_once(&self, addr: VAddr, level: u8) -> PtQuery<Meta> {
        let vpn = addr.0 >> (OFFSET_BITS + level as usize * PT_LEVEL_BITS);
        let pte = self.0[vpn & ((1 << PT_LEVEL_BITS) - 1)];
        if !pte.is_valid() {
            PtQuery::Invalid
        } else if pte.is_leaf() {
            PtQuery::Leaf(pte)
        } else {
            PtQuery::SubTable(pte.ppn())
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
