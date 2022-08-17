use crate::{VmFlags, VmMeta, PPN};
use core::marker::PhantomData;

/// 页表项。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

    /// 如果页表项指向的页可读，返回 `true`。
    #[inline]
    pub fn is_readable(self) -> bool {
        Meta::is_readable(self.0)
    }

    /// 如果页表项指向的页可写，返回 `true`。
    #[inline]
    pub fn is_writable(self) -> bool {
        Meta::is_writable(self.0)
    }

    /// 如果页表项指向的页可执行，返回 `true`。
    #[inline]
    pub fn is_executable(self) -> bool {
        Meta::is_executable(self.0)
    }

    /// 如果页表项指向的页用户态可访问，返回 `true`。
    #[inline]
    pub fn is_user(self) -> bool {
        Meta::is_user(self.0)
    }

    /// 如果页表项指向全局页，返回 `true`。
    #[inline]
    pub fn is_global(self) -> bool {
        Meta::is_global(self.0)
    }

    /// 如果页表项指向的页已访问，返回 `true`。
    #[inline]
    pub fn is_accessed(self) -> bool {
        Meta::is_accessed(self.0)
    }

    /// 如果页表项指向的页已写，返回 `true`。
    #[inline]
    pub fn is_dirty(self) -> bool {
        Meta::is_dirty(self.0)
    }

    /// 修改页表项指向的物理页。
    #[inline]
    pub fn set_ppn(&mut self, ppn: PPN<Meta>) {
        Meta::clear_ppn(&mut self.0);
        Meta::set_ppn(&mut self.0, ppn);
    }

    /// 取出页表项属性。
    #[inline]
    pub fn flags(mut self) -> VmFlags<Meta> {
        Meta::clear_ppn(&mut self.0);
        VmFlags::new(self.0)
    }
}
