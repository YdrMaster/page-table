use crate::{mask, VmMeta};
use core::{
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign},
};

/// 页号。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
    /// 最大物理页号。
    pub const MAX: Self = Self::new(mask(Meta::P_ADDR_BITS - Meta::PAGE_BITS));
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
        Meta::LEVEL_BITS
            .iter()
            .take(level)
            .fold(self.0, |bits, it| bits >> it)
            & mask(Meta::LEVEL_BITS[level])
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
}

/// 虚拟地址。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct VAddr<Meta: VmMeta>(usize, PhantomData<Meta>);

impl<Meta: VmMeta> VAddr<Meta> {
    /// 将一个地址值转换为虚拟地址意味着允许虚存方案根据实际情况裁剪地址值。
    /// 超过虚址范围的地址会被裁剪。
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value, PhantomData)
    }

    /// 将虚地址转化为任意指针。
    ///
    /// # Safety
    ///
    /// 调用者需要确保虚地址在当前地址空间中。
    #[inline]
    pub const unsafe fn as_ptr<T>(self) -> *const T {
        self.0 as _
    }

    /// 将虚地址转化为任意可变指针。
    ///
    /// # Safety
    ///
    /// 调用者需要确保虚地址在当前地址空间中。
    #[inline]
    pub unsafe fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as _
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
