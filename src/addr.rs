use crate::{mask, PAGE_BITS, PT_LEVEL_BITS, P_ADDR_BITS};
use core::{
    marker::PhantomData,
    ops::{Add, AddAssign},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct PageNumber<S: Space>(usize, PhantomData<S>);

pub trait Space {}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Physical;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Virtual;

impl Space for Physical {}
impl Space for Virtual {}

impl<S: Space> PageNumber<S> {
    /// 页号零。
    pub const ZERO: Self = Self::new(0);

    /// 最小页号。
    pub const MIN: Self = Self::ZERO;

    #[inline]
    pub const fn new(n: usize) -> Self {
        Self(n, PhantomData)
    }

    #[inline]
    pub const fn val(self) -> usize {
        self.0
    }
}

impl<S: Space> Add<usize> for PageNumber<S> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self {
        Self(self.0.wrapping_add(rhs), PhantomData)
    }
}

impl<S: Space> AddAssign<usize> for PageNumber<S> {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 = self.0.wrapping_add(rhs);
    }
}

/// 物理页号。
pub type PPN = PageNumber<Physical>;

/// 虚拟页号。
pub type VPN = PageNumber<Virtual>;

impl PPN {
    /// 最大物理页号。
    pub const MAX: Self = Self(mask(P_ADDR_BITS - PAGE_BITS), PhantomData);
}

impl VPN {
    /// 虚页的起始地址。
    #[inline]
    pub const fn base(self) -> VAddr {
        VAddr(self.0 << PAGE_BITS)
    }

    /// 虚页在 `level` 级页表中的位置。
    #[inline]
    pub const fn index_in(self, level: usize) -> usize {
        (self.0 >> (level * PT_LEVEL_BITS)) & mask(PT_LEVEL_BITS)
    }

    /// 虚页在 `level` 级页上的偏移。
    #[inline]
    pub const fn offset_in(self, level: usize) -> usize {
        self.0 & mask(level * PT_LEVEL_BITS)
    }
}

/// 虚拟地址。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(transparent)]
pub struct VAddr(usize);

impl VAddr {
    /// 将一个地址值转换为虚拟地址意味着允许虚存方案根据实际情况裁剪地址值。
    /// 超过虚址范围的地址会被裁剪。
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value)
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
    pub const fn floor(self) -> VPN {
        VPN::new(self.0 >> PAGE_BITS)
    }

    /// 不包括这个虚地址的最前页的页号。
    #[inline]
    pub const fn ceil(self) -> VPN {
        VPN::new((self.0 + mask(PAGE_BITS)) >> PAGE_BITS)
    }
}

impl From<usize> for VAddr {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl<T> From<&T> for VAddr {
    #[inline]
    fn from(value: &T) -> Self {
        Self(value as *const _ as _)
    }
}
