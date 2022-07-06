use crate::{MmuFlags, MmuMeta, PPN};
use core::marker::PhantomData;

/// 页表项。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct Pte<Meta: MmuMeta>(pub usize, pub(crate) PhantomData<Meta>);

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
        MmuFlags::new(self.0)
    }
}
