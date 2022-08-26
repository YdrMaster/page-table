mod fmt;
mod pos;
mod shuttle;

use crate::{Pte, VmMeta, VPN};
use core::ops::{Index, IndexMut, Range};

pub use pos::Pos;
pub use shuttle::{Decorator, PageTableShuttle, Update, Visitor};

/// 页表。
///
/// 不持有页表的所有权，因为页表总是在一些物理页帧上。
pub struct PageTable<Meta: VmMeta> {
    mem: &'static mut [Pte<Meta>],
    base: VPN<Meta>,
    level: usize,
}

impl<Meta: VmMeta> PageTable<Meta> {
    /// 从指向第一个页表项的指针创建页表。
    ///
    /// # Safety
    ///
    /// 同 [from_raw_parts_mut](core::slice::from_raw_parts_mut).
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut Pte<Meta>, base: VPN<Meta>, level: usize) -> Self {
        Self {
            // 显然需要 level <= Meta::MAX_LEVEL
            mem: core::slice::from_raw_parts_mut(ptr, 1 << Meta::LEVEL_BITS[level]),
            base: base.floor(level),
            level,
        }
    }

    /// 获取指向第一个页表项的指针。
    #[inline]
    pub const fn as_ptr(&self) -> *const Pte<Meta> {
        self.mem.as_ptr()
    }

    /// 获取页表级别。
    #[inline]
    pub const fn level(&self) -> usize {
        self.level
    }

    /// 获取页表容纳的虚页号范围。
    #[inline]
    pub fn range(&self) -> Range<VPN<Meta>> {
        self.base..self.base + Meta::pages_in_table(self.level)
    }
}

impl<Meta: VmMeta> Index<usize> for PageTable<Meta> {
    type Output = Pte<Meta>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.mem[index]
    }
}

impl<Meta: VmMeta> IndexMut<usize> for PageTable<Meta> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.mem[index]
    }
}
