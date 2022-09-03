use crate::{mask, VmMeta};
use core::{
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign, Range},
};

/// 页号。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PageNumber<Meta: VmMeta, S: Space>(usize, PhantomData<Meta>, PhantomData<S>);

/// 地址空间标记。
pub trait Space: Clone + Copy + PartialEq + Eq + PartialOrd + Ord + fmt::Debug {}

/// 物理地址空间。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Physical;

/// 虚地址空间。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Virtual;

impl Space for Physical {}
impl Space for Virtual {}

impl<Meta: VmMeta, S: Space> PageNumber<Meta, S> {
    /// 页号零。
    pub const ZERO: Self = Self::new(0);

    /// 最小页号。
    pub const MIN: Self = Self::ZERO;

    /// 新建一个页号。
    #[inline]
    pub const fn new(n: usize) -> Self {
        Self(n, PhantomData, PhantomData)
    }

    /// 获取页号值。
    #[inline]
    pub const fn val(self) -> usize {
        self.0
    }
}

impl<Meta: VmMeta, S: Space> Add<usize> for PageNumber<Meta, S> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self::new(self.0.wrapping_add(rhs))
    }
}

impl<Meta: VmMeta, S: Space> AddAssign<usize> for PageNumber<Meta, S> {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 = self.0.wrapping_add(rhs);
    }
}

/// 物理页号。
pub type PPN<Meta> = PageNumber<Meta, Physical>;

/// 虚拟页号。
pub type VPN<Meta> = PageNumber<Meta, Virtual>;

impl<Meta: VmMeta> PPN<Meta> {
    /// 无效的物理页号，作为 NULL 使用。
    ///
    /// 显然，物理页号不可能和 usize 一样长，所以可以这样操作。
    pub const INVALID: Self = Self::new(1 << (Meta::P_ADDR_BITS - Meta::PAGE_BITS));

    /// 最大物理页号。
    pub const MAX: Self = Self::new(Self::INVALID.val() - 1);
}

impl<Meta: VmMeta> fmt::Debug for PPN<Meta> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PPN({:#x})", self.0)
    }
}

impl<Meta: VmMeta> VPN<Meta> {
    /// 最大虚拟页号。
    pub const MAX: Self = Self::new(mask(Meta::V_ADDR_BITS - Meta::PAGE_BITS));

    /// 虚页的起始地址。
    #[inline]
    pub const fn base(self) -> VAddr<Meta> {
        VAddr::new(self.0 << Meta::PAGE_BITS)
    }

    /// 虚页在 `level` 级页表中的位置。
    #[inline]
    pub fn index_in(self, level: usize) -> usize {
        (self.0 >> Self::bits_until(level)) & mask(Meta::LEVEL_BITS[level])
    }

    /// 包含这个虚页的 `level` 级页表起始地址。
    #[inline]
    pub fn floor(self, level: usize) -> Self {
        let bits = Self::bits_until(level);
        Self::new(self.0 & !mask(bits))
    }

    /// 不包含这个虚页的 `level` 级页表起始地址。
    #[inline]
    pub fn ceil(self, level: usize) -> usize {
        let bits = Self::bits_until(level);
        (self.0 + mask(bits)) >> bits
    }

    /// 包含这个虚页的 `level` 级页表容纳的虚页范围。
    #[inline]
    pub fn vaddr_range(self, level: usize) -> Range<VAddr<Meta>> {
        let base = self.base();
        base..base + Meta::bytes_in_page(level)
    }

    /// 虚页的对齐级别，使虚页在页表中序号为 0 的最高等级页表的级别。
    #[inline]
    pub fn align_level(self) -> usize {
        let mut n = self.0;
        for (i, bits) in Meta::LEVEL_BITS[..Meta::MAX_LEVEL].iter().rev().enumerate() {
            if n & mask(*bits) != 0 {
                return i;
            }
            n >>= bits;
        }
        Meta::MAX_LEVEL
    }

    /// `level` 级页表中页号的总位数。
    #[inline]
    fn bits_until(level: usize) -> usize {
        Meta::LEVEL_BITS[..level].iter().sum::<usize>()
    }
}

impl<Meta: VmMeta> fmt::Debug for VPN<Meta> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VPN({:#x})", self.0)
    }
}

/// 一个可能无效的物理页号。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MaybeInvalidPPN<Meta: VmMeta>(PPN<Meta>);

impl<Meta: VmMeta> MaybeInvalidPPN<Meta> {
    /// 从一个物理页号新建。
    #[inline]
    pub const fn new(n: PPN<Meta>) -> Self {
        Self(n)
    }

    /// 新建一个无效物理页号。
    #[inline]
    pub const fn invalid() -> Self {
        Self(PPN::INVALID)
    }

    /// 取出物理页号。
    #[inline]
    pub const fn get(&self) -> Option<PPN<Meta>> {
        if self.0.val() > PPN::<Meta>::MAX.val() {
            None
        } else {
            Some(self.0)
        }
    }
}

/// 虚拟地址。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VAddr<Meta: VmMeta>(usize, PhantomData<Meta>);

impl<Meta: VmMeta> Add<usize> for VAddr<Meta> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self::new(self.0.wrapping_add(rhs))
    }
}

impl<Meta: VmMeta> AddAssign<usize> for VAddr<Meta> {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 = self.0.wrapping_add(rhs);
    }
}

impl<Meta: VmMeta> VAddr<Meta> {
    const IGNORED_MASK: usize = mask(Meta::V_ADDR_BITS - 1);

    /// 将一个地址值转换为虚拟地址意味着允许虚存方案根据实际情况裁剪地址值。
    /// 超过虚址范围的地址会被裁剪。
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value & mask(Meta::V_ADDR_BITS), PhantomData)
    }

    /// 虚地址值。
    #[inline]
    pub const fn val(self) -> usize {
        if self.0 <= Self::IGNORED_MASK {
            self.0
        } else {
            self.0 | !Self::IGNORED_MASK
        }
    }

    /// 将虚地址转化为任意指针。
    ///
    /// # Safety
    ///
    /// 调用者需要确保虚地址在当前地址空间中。
    #[inline]
    pub const unsafe fn as_ptr<T>(self) -> *const T {
        self.val() as _
    }

    /// 将虚地址转化为任意可变指针。
    ///
    /// # Safety
    ///
    /// 调用者需要确保虚地址在当前地址空间中。
    #[inline]
    pub unsafe fn as_mut_ptr<T>(self) -> *mut T {
        self.val() as _
    }

    /// 包括这个虚地址最后页的页号。
    #[inline]
    pub const fn floor(self) -> VPN<Meta> {
        VPN::new(self.0 >> Meta::PAGE_BITS)
    }

    /// 不包括这个虚地址的最前页的页号。
    #[inline]
    pub const fn ceil(self) -> VPN<Meta> {
        VPN::new((self.0 + mask(Meta::PAGE_BITS)) >> Meta::PAGE_BITS)
    }

    /// 页内偏移。
    #[inline]
    pub const fn offset(self) -> usize {
        self.0 & mask(Meta::PAGE_BITS)
    }
}

impl<Meta: VmMeta> From<usize> for VAddr<Meta> {
    #[inline]
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl<Meta: VmMeta, T> From<&T> for VAddr<Meta> {
    #[inline]
    fn from(value: &T) -> Self {
        Self::new(value as *const _ as _)
    }
}

impl<Meta: VmMeta> fmt::Debug for VAddr<Meta> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VAddr({:#x})", self.0)
    }
}

#[test]
fn test_index_in() {
    use crate::test_meta::Sv39;

    for i in 0..=Sv39::MAX_LEVEL {
        let vpn = VPN::<Sv39>::new(1 << (i * 9));
        for j in 0..=Sv39::MAX_LEVEL {
            assert_eq!(vpn.index_in(j), if i == j { 1 } else { 0 });
        }
    }
}

#[test]
fn test_align_level() {
    use crate::test_meta::Sv39;

    assert_eq!(VPN::<Sv39>::new(1).align_level(), 0);
    assert_eq!(VPN::<Sv39>::new(1 << 9).align_level(), 1);
    assert_eq!(VPN::<Sv39>::new(1 << 18).align_level(), 2);
    assert_eq!(VPN::<Sv39>::new(0).align_level(), 2);
}
