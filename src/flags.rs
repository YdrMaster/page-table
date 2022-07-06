use crate::{MmuMeta, Pte, PPN};
use core::marker::PhantomData;

/// MMU 属性。
///
/// MMU 属性一定完全包含在页表项中，所以独立的 MMU 属性实现为一个无法获取地址的页表项。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MmuFlags<Meta: MmuMeta>(pub usize, PhantomData<Meta>);

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
