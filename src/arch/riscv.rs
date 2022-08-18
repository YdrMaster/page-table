cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        /// 32 位 RISC-V 物理地址位数。
        const P_ADDR_BITS: usize = 34;
        /// RISC-V Sv32 VM Mode.
        pub type Sv32 = Sv<2>;
    } else if #[cfg(target_pointer_width = "64")] {
        /// 64 位 RISC-V 物理地址位数。
        const P_ADDR_BITS: usize = 56;
        /// RISC-V Sv39 VM Mode.
        pub type Sv39 = Sv<3>;
        /// RISC-V Sv48 VM Mode.
        pub type Sv48 = Sv<4>;
        /// RISC-V Sv57 VM Mode.
        pub type Sv57 = Sv<5>;
    }
}

/// RISC-V 标准定义的虚存方案。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Sv<const N: usize>;

impl<const N: usize> crate::MmuMeta for Sv<N> {
    const P_ADDR_BITS: usize = P_ADDR_BITS;
    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: &'static [usize] = &[pt_level_bits(Self::PAGE_BITS); N];
    const PPN_POS: usize = 10;

    #[inline]
    fn is_leaf(value: usize) -> bool {
        const MASK: usize = 0b1110;
        value & MASK != 0
    }

    fn fmt_flags(f: &mut core::fmt::Formatter, flags: usize) -> core::fmt::Result {
        const FLAGS: [u8; 8] = [b'V', b'R', b'W', b'X', b'U', b'G', b'A', b'D'];
        for (i, w) in FLAGS.iter().enumerate().rev() {
            if (flags >> i) & 1 == 1 {
                write!(f, "{}", *w as char)?;
            } else {
                write!(f, "_")?;
            }
        }
        Ok(())
    }
}

#[inline]
const fn pt_level_bits(page_bits: usize) -> usize {
    page_bits - core::mem::size_of::<usize>().trailing_zeros() as usize
}

#[cfg(target_pointer_width = "32")]
mod assertions {
    use super::*;
    use crate::{MmuMeta, VmMeta};
    use static_assertions::const_assert_eq;

    const_assert_eq!(Sv32::V_ADDR_BITS, 32);
    const_assert_eq!(Sv32::MAX_LEVEL, 1);
    const_assert_eq!(Sv32::PAGE_BITS, 12);
}

#[cfg(target_pointer_width = "64")]
mod assertions {
    use super::*;
    use crate::{MmuMeta, VmMeta};
    use static_assertions::const_assert_eq;

    const_assert_eq!(Sv39::V_ADDR_BITS, 39);
    const_assert_eq!(Sv48::V_ADDR_BITS, 48);
    const_assert_eq!(Sv57::V_ADDR_BITS, 57);

    const_assert_eq!(Sv39::MAX_LEVEL, 2);
    const_assert_eq!(Sv48::MAX_LEVEL, 3);
    const_assert_eq!(Sv57::MAX_LEVEL, 4);

    const_assert_eq!(Sv39::PAGE_BITS, 12);
    const_assert_eq!(Sv48::PAGE_BITS, 12);
    const_assert_eq!(Sv57::PAGE_BITS, 12);
}
