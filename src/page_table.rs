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
    pub const ZERO: Self = Self([Pte::ZERO; ENTRIES_PER_TABLE]);

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        !self.0.iter().any(|pte| pte.is_valid())
    }

    #[inline]
    pub fn as_ptr(&self) -> *const Pte<Meta> {
        self.0.as_ptr()
    }

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

    pub fn query(&self, addr: VAddr, level: u8) -> PtQuery<Meta> {
        let mut idx = addr.0 >> OFFSET_BITS;
        for _ in 0..level {
            idx >>= PT_LEVEL_BITS;
        }
        idx &= (1 << PT_LEVEL_BITS) - 1;
        let pte = self.0[idx];
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
