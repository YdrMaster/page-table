cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        const P_ADDR_BITS: usize = 34;
        /// RISC-V Sv32 VM Mode.
        pub type Sv32 = Sv<32>;
    } else if #[cfg(target_pointer_width = "64")] {
        const P_ADDR_BITS: usize = 56;
        /// RISC-V Sv39 VM Mode.
        pub type Sv39 = Sv<39>;
        /// RISC-V Sv48 VM Mode.
        pub type Sv48 = Sv<48>;
        /// RISC-V Sv57 VM Mode.
        pub type Sv57 = Sv<57>;
    }
}

/// RISC-V 标准定义的虚存方案。
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Sv<const N: usize>;

impl<const N: usize> crate::MmuMeta for Sv<N> {
    const P_ADDR_BITS: usize = P_ADDR_BITS;
    const V_ADDR_BITS: usize = N;
    const PPN_BASE: usize = 10;
    const FLAG_POS_V: usize = 0;
    const FLAG_POS_R: usize = 1;
    const FLAG_POS_W: usize = 2;
    const FLAG_POS_X: usize = 3;
    const FLAG_POS_U: usize = 4;
    const FLAG_POS_G: usize = 5;
    const FLAG_POS_A: usize = 6;
    const FLAG_POS_D: usize = 7;

    #[inline]
    fn is_leaf(value: usize) -> bool {
        const MASK: usize = 0b1110;
        value & MASK != 0
    }
}
