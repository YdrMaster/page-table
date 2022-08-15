use crate::{OFFSET_BITS, PT_LEVEL_BITS};

/// 物理页号。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct PPN(pub usize);

/// 虚拟页号。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct VPN(pub usize);

impl VPN {
    /// 虚页的起始地址。
    #[inline]
    pub const fn base(self) -> VAddr {
        VAddr(self.0 << OFFSET_BITS)
    }

    /// 虚页在 `level` 级页表中的位置。
    #[inline]
    pub const fn index(self, level: usize) -> usize {
        const MASK: usize = (1 << PT_LEVEL_BITS) - 1;
        (self.0 >> level * PT_LEVEL_BITS) & MASK
    }
}

/// 虚拟地址。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
    #[allow(unsafe_code)]
    #[inline]
    pub unsafe fn as_ptr<T>(self) -> *const T {
        self.0 as _
    }

    /// 将虚地址转化为任意可变指针。
    ///
    /// # Safety
    ///
    /// 调用者需要确保虚地址在当前地址空间中。
    #[allow(unsafe_code)]
    #[inline]
    pub unsafe fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as _
    }

    /// 包括这个虚地址最后页的页号。
    #[inline]
    pub const fn floor(self) -> VPN {
        VPN(self.0 >> OFFSET_BITS)
    }

    /// 不包括这个虚地址的最前页的页号。
    #[inline]
    pub const fn ceil(self) -> VPN {
        VPN((self.0 + (1 << OFFSET_BITS) - 1) >> OFFSET_BITS)
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
