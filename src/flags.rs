use crate::{Pte, VmMeta, PPN};
use core::{
    marker::PhantomData,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

/// 页表项属性。
///
/// 页表项属性一定完全包含在页表项中，所以独立的页表项属性实现为一个无法获取地址的页表项。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct VmFlags<Meta: VmMeta>(usize, PhantomData<Meta>);

impl<Meta: VmMeta> VmFlags<Meta> {
    /// 将 `raw` 整数转化为一个页表项属性。
    ///
    /// # Safety
    ///
    /// 调用者需要保证 `raw` 里不表示属性的位全是零。
    #[inline]
    pub const unsafe fn from_raw(raw: usize) -> Self {
        Self(raw, PhantomData)
    }

    /// 取出值。
    #[inline]
    pub const fn val(self) -> usize {
        self.0
    }

    /// 判断是否包含所有指定的位。
    #[inline]
    pub const fn contains(self, flags: VmFlags<Meta>) -> bool {
        self.0 & flags.0 == flags.0
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
    pub fn valid(self) -> bool {
        Meta::is_valid(self.0)
    }

    /// 构造具有 `self` 页表项属性，并指向 `ppn` 物理页的页表项。
    #[inline]
    pub fn build_pte(mut self, ppn: PPN<Meta>) -> Pte<Meta> {
        Meta::set_ppn(&mut self.0, ppn);
        Pte(self.0, PhantomData)
    }
}

impl<Meta: VmMeta> BitAnd for VmFlags<Meta> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, PhantomData)
    }
}

impl<Meta: VmMeta> BitOr for VmFlags<Meta> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, PhantomData)
    }
}

impl<Meta: VmMeta> BitXor for VmFlags<Meta> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0, PhantomData)
    }
}

impl<Meta: VmMeta> BitAndAssign for VmFlags<Meta> {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl<Meta: VmMeta> BitOrAssign for VmFlags<Meta> {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl<Meta: VmMeta> BitXorAssign for VmFlags<Meta> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}
