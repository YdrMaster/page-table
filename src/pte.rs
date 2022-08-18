use crate::{VmFlags, VmMeta, PPN};
use core::{fmt, marker::PhantomData};

/// 页表项。
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Pte<Meta: VmMeta>(pub usize, pub(crate) PhantomData<Meta>);

impl<Meta: VmMeta> Pte<Meta> {
    /// 空白页表项。
    pub const ZERO: Self = Self(0, PhantomData);

    /// 获取页表项指向的物理页号。
    #[inline]
    pub fn ppn(self) -> PPN<Meta> {
        Meta::ppn(self.0)
    }

    /// 如果页表项指向一个页而非子页表，返回 `true`。
    #[inline]
    pub fn is_leaf(self) -> bool {
        Meta::is_leaf(self.0)
    }

    /// 如果页表项指向一个非 0 级的页，返回 `true`。
    #[inline]
    pub fn is_huge(self, level: usize) -> bool {
        Meta::is_huge(self.0, level)
    }

    /// 如果页表项有效，返回 `true`。
    #[inline]
    pub fn is_valid(self) -> bool {
        Meta::is_valid(self.0)
    }

    /// 取出页表项属性。
    #[inline]
    pub fn flags(mut self) -> VmFlags<Meta> {
        Meta::clear_ppn(&mut self.0);
        unsafe { VmFlags::from_raw(self.0) }
    }
}

impl<Meta: VmMeta> fmt::Debug for Pte<Meta> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pte(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}
