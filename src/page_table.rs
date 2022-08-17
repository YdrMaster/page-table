use crate::{const_sum, Pte, VAddr, VmMeta, PPN};
use core::ops::{Index, IndexMut};

/// 页表。
#[repr(C, align(4096))]
pub struct PageTable<Meta: VmMeta>(&'static mut [Pte<Meta>]);

impl<Meta: VmMeta> PageTable<Meta> {
    /// 从指向第一个页表项的指针创建页表。
    ///
    /// # Safety
    ///
    /// 同 [from_raw_parts_mut](core::slice::from_raw_parts_mut).
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut Pte<Meta>, level: usize) -> Self {
        let len = 1 << Meta::LEVEL_BITS[level + 1];
        Self(core::slice::from_raw_parts_mut(ptr, len))
    }

    /// 获取指向第一个页表项的指针。
    #[inline]
    pub const fn as_ptr(&self) -> *const Pte<Meta> {
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
        vaddr: VAddr<Meta>,
        entry: Pte<Meta>,
        level: usize,
    ) -> Result<(), EntryError> {
        if level > Meta::MAX_LEVEL {
            Err(EntryError::InvalidLevel)?;
        }
        let page_align = const_sum(0, &Meta::LEVEL_BITS[1..][..level]);
        if entry.is_huge(level) && (entry.ppn().val().trailing_zeros() as usize) < page_align {
            Err(EntryError::LeafMisaligned)?;
        }
        self.0[vaddr.floor().index_in(level)] = entry;
        Ok(())
    }

    /// 查询页表一次。
    #[inline]
    pub fn query_once(&self, vaddr: VAddr<Meta>, level: usize) -> PtQuery<Meta> {
        self.0[vaddr.floor().index_in(level)].into()
    }

    /// 返回一个遍历并擦除页表的迭代器。
    #[inline]
    pub fn erase(&mut self) -> Eraser<'_, Meta> {
        Eraser(self, 0)
    }
}

impl<Meta: VmMeta> Index<usize> for PageTable<Meta> {
    type Output = Pte<Meta>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<Meta: VmMeta> IndexMut<usize> for PageTable<Meta> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

/// 查询结果。
pub enum PtQuery<Meta: VmMeta> {
    /// 虚存对应的页表项不存在。
    Invalid,
    /// 下一级页表的物理页号。
    SubTable(PPN<Meta>),
    /// 页表项。
    Leaf(Pte<Meta>),
}

impl<Meta: VmMeta> From<Pte<Meta>> for PtQuery<Meta> {
    #[inline]
    fn from(pte: Pte<Meta>) -> Self {
        if !pte.is_valid() {
            Self::Invalid
        } else if pte.is_leaf() {
            Self::Leaf(pte)
        } else {
            Self::SubTable(pte.ppn())
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EntryError {
    InvalidLevel,
    LeafMisaligned,
}

/// 擦除迭代器。
pub struct Eraser<'a, Meta: VmMeta>(&'a mut PageTable<Meta>, usize);

impl<'a, Meta: VmMeta> Iterator for Eraser<'a, Meta> {
    type Item = PtQuery<Meta>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ans) = self.0 .0.get_mut(self.1) {
            let ans = core::mem::replace(ans, Pte::ZERO);
            match PtQuery::from(ans) {
                PtQuery::Invalid => {}
                query => return Some(query),
            }
        }
        None
    }
}
