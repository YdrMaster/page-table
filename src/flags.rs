use crate::{MmuMeta, Pte, PPN};
use core::marker::PhantomData;

/// 页表项属性。
///
/// 页表项属性一定完全包含在页表项中，所以独立的页表项属性实现为一个无法获取地址的页表项。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MmuFlags<Meta: MmuMeta>(pub usize, PhantomData<Meta>);

impl<Meta: MmuMeta> MmuFlags<Meta> {
    /// 所有位为 0 的页表项属性。
    pub const ZERO: Self = Self(0, PhantomData);

    /// 将 `raw` 整数转化为一个页表项属性。
    #[inline]
    pub const fn new(raw: usize) -> Self {
        Self(raw, PhantomData)
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

    /// 构造具有 `self` 页表项属性，并指向 `ppn` 物理页的页表项。
    #[inline]
    pub fn build_pte(mut self, ppn: PPN) -> Pte<Meta> {
        Meta::set_ppn(&mut self.0, ppn);
        Pte(self.0, PhantomData)
    }
}
