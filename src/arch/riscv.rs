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
    const LEVEL_BITS: &'static [usize] = level_bits::<N>();
    const FLAG_POS_V: usize = 0;
    const FLAG_POS_R: usize = 1;
    const FLAG_POS_W: usize = 2;
    const FLAG_POS_X: usize = 3;
    const FLAG_POS_U: usize = 4;
    const FLAG_POS_G: usize = 5;
    const FLAG_POS_A: usize = 6;
    const FLAG_POS_D: usize = 7;
    const PPN_POS: usize = 10;

    #[inline]
    fn is_leaf(value: usize) -> bool {
        const MASK: usize = 0b1110;
        value & MASK != 0
    }
}

const fn level_bits<const N: usize>() -> &'static [usize] {
    match N {
        2 => &[12, 10, 10],
        3 => &[12, 9, 9, 9],
        4 => &[12, 9, 9, 9, 9],
        5 => &[12, 9, 9, 9, 9, 9],
        _ => unreachable!(),
    }
}

#[cfg(target_pointer_width = "32")]
mod assertions {
    use super::*;
    use crate::VmMeta;
    use static_assertions::const_assert_eq;

    const_assert_eq!(Sv32::V_ADDR_BITS, 32);
    const_assert_eq!(Sv32::MAX_LEVEL, 2);
    const_assert_eq!(Sv32::PAGE_BITS, 12);
}

#[cfg(target_pointer_width = "64")]
mod assertions {
    use super::*;
    use crate::VmMeta;
    use static_assertions::const_assert_eq;

    const_assert_eq!(Sv39::V_ADDR_BITS, 39);
    const_assert_eq!(Sv48::V_ADDR_BITS, 48);
    const_assert_eq!(Sv57::V_ADDR_BITS, 57);

    const_assert_eq!(Sv39::MAX_LEVEL, 3);
    const_assert_eq!(Sv48::MAX_LEVEL, 4);
    const_assert_eq!(Sv57::MAX_LEVEL, 5);

    const_assert_eq!(Sv39::PAGE_BITS, 12);
    const_assert_eq!(Sv48::PAGE_BITS, 12);
    const_assert_eq!(Sv57::PAGE_BITS, 12);
}
