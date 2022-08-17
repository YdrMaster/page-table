use crate::{const_sum, mask, VmMeta, P_ADDR_BITS};
use core::{
    fmt,
    marker::PhantomData,
    ops::{Add, AddAssign},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PageNumber<Meta: VmMeta, S: Space>(usize, PhantomData<Meta>, PhantomData<S>);

pub trait Space: Clone + Copy + PartialEq + Eq + PartialOrd + Ord + fmt::Debug {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Physical;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Virtual;

impl Space for Physical {}
impl Space for Virtual {}

impl<Meta: VmMeta, S: Space> PageNumber<Meta, S> {
    /// 页号零。
    pub const ZERO: Self = Self::new(0);

    /// 最小页号。
    pub const MIN: Self = Self::ZERO;

    #[inline]
    pub const fn new(n: usize) -> Self {
        Self(n, PhantomData, PhantomData)
    }

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
    pub const MAX: Self = Self::new(mask(P_ADDR_BITS - Meta::PAGE_BITS));
}

impl<Meta: VmMeta> VPN<Meta> {
    /// 虚页的起始地址。
    #[inline]
    pub const fn base(self) -> VAddr<Meta> {
        VAddr::new(self.0 << Meta::PAGE_BITS)
    }

    /// 虚页在 `level` 级页表中的位置。
    #[inline]
    pub fn index_in(self, level: usize) -> usize {
        let base = const_sum(0, &Meta::LEVEL_BITS[1..][..level]);
        (self.0 >> base) & mask(Meta::LEVEL_BITS[level + 1])
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
