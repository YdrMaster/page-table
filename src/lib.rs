//! x

#![no_std]
#![deny(warnings, unstable_features, missing_docs)]

mod addr;
mod flags;
mod pte;
mod table;

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {
        #[path = "arch/riscv.rs"]
        mod arch;
    } else if #[cfg(target_arch = "aarch64")] {
        #[path = "arch/arm.rs"]
        mod arch;
    } else if #[cfg(target_arch = "x86_64")] {
        #[path = "arch/x86.rs"]
        mod arch;
    } else {
        compile_error!("Unsupported architecture");
    }
}

pub use addr::*;
pub use arch::*;
pub use flags::VmFlags;
pub use pte::Pte;
pub use table::*;

/// 地址转换单元元数据。
pub trait MmuMeta {
    /// 物理地址位数，用于计算物理页号形式。
    const P_ADDR_BITS: usize;

    /// 页内偏移的位数。
    const PAGE_BITS: usize;

    /// 各级页内虚地址位数位数。
    const LEVEL_BITS: &'static [usize];

    /// 物理页号在 PTE 中的位置。
    const PPN_POS: usize;

    /// 判断页表项是否有效。
    #[inline]
    fn is_valid(value: usize) -> bool {
        // 一般都用最低位表示页表有效
        value & 1 == 1
    }

    /// 如果页表项指向物理页，则返回 `true`。
    ///
    /// # NOTE
    ///
    /// 为了分散开销，这个方法的实现不会判断 PTE 是否 valid。
    fn is_leaf(value: usize) -> bool;
}

/// 页式虚存元数据。
pub trait VmMeta: 'static + MmuMeta + Copy + Ord + core::hash::Hash + core::fmt::Debug {
    /// 虚拟页号位数，用于裁剪或扩展正确的虚址。
    const V_ADDR_BITS: usize = Self::PAGE_BITS + const_sum(0, Self::LEVEL_BITS);

    /// 页表最大级别。
    const MAX_LEVEL: usize = Self::LEVEL_BITS.len() - 1;

    /// 页表项中的物理页号掩码。
    const PPN_MASK: usize = ppn_mask::<Self>();

    /// `level` 级页表容纳的总页数。
    #[inline]
    fn pages_in_table(level: usize) -> usize {
        1 << Self::LEVEL_BITS[..=level].iter().sum::<usize>()
    }

    /// `level` 级页表容纳的总字节数。
    #[inline]
    fn bytes_in_table(level: usize) -> usize {
        1 << (Self::LEVEL_BITS[..=level].iter().sum::<usize>() + Self::PAGE_BITS)
    }

    /// `level` 级页容纳的总字节数。
    #[inline]
    fn bytes_in_page(level: usize) -> usize {
        1 << (Self::LEVEL_BITS[..level].iter().sum::<usize>() + Self::PAGE_BITS)
    }

    /// 判断页表项指向的是一个大于 0 级（4 kiB）的物理页。
    ///
    /// # NOTE
    ///
    /// 为了零开销抽象，这个方法的实现可能不会判断 PTE 是否 valid。
    #[inline]
    fn is_huge(value: usize, level: usize) -> bool {
        level != 0 && Self::is_leaf(value)
    }

    /// 从 PTE 中获得 PPN。
    #[inline]
    fn ppn(value: usize) -> PPN<Self> {
        PPN::new((value & Self::PPN_MASK) >> Self::PPN_POS)
    }

    /// 设置页表项的 ppn。
    #[inline]
    fn set_ppn(value: &mut usize, ppn: PPN<Self>) {
        *value |= (ppn.val() << Self::PPN_POS) & Self::PPN_MASK;
    }

    /// 清除页表项中的 ppn。
    #[inline]
    fn clear_ppn(value: &mut usize) {
        *value &= !Self::PPN_MASK;
    }
}

/// 自动实现。
impl<T: 'static + MmuMeta + Copy + Ord + core::hash::Hash + core::fmt::Debug> VmMeta for T {}

/// 生成一个 `bits` 位的掩码。
#[inline]
const fn mask(bits: usize) -> usize {
    (1 << bits) - 1
}

/// 计算 pte 中 ppn 的掩码。
#[inline]
const fn ppn_mask<Meta: MmuMeta>() -> usize {
    let m0: usize = !mask(Meta::PPN_POS);
    let m1: usize = mask(Meta::PPN_POS + Meta::P_ADDR_BITS - Meta::PAGE_BITS);
    m0 & m1
}

/// 递归求和，可以用在编译期。
#[inline]
const fn const_sum(val: usize, bits: &[usize]) -> usize {
    match bits {
        [] => val,
        [n, tail @ ..] => const_sum(val + *n, tail),
    }
}

#[cfg(test)]
mod test_meta {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub(crate) struct Sv39;

    impl super::MmuMeta for Sv39 {
        const P_ADDR_BITS: usize = 56;
        const PAGE_BITS: usize = 12;
        const LEVEL_BITS: &'static [usize] = &[9; 3];
        const PPN_POS: usize = 10;

        #[inline]
        fn is_leaf(value: usize) -> bool {
            const MASK: usize = 0b1110;
            value & MASK != 0
        }
    }

    #[test]
    fn test_pages() {
        use super::VmMeta;

        assert_eq!(Sv39::pages_in_table(0), 512);
        assert_eq!(Sv39::pages_in_table(1), 512 * 512);
        assert_eq!(Sv39::pages_in_table(2), 512 * 512 * 512);

        assert_eq!(Sv39::bytes_in_table(0), 4096 * 512);
        assert_eq!(Sv39::bytes_in_table(1), 4096 * 512 * 512);
        assert_eq!(Sv39::bytes_in_table(2), 4096 * 512 * 512 * 512);
    }
}
