use crate::{MmuMeta, Pte, OFFSET_BITS, PPN};

cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        const PADDR_BITS: usize = 34;
        pub type Sv32 = Sv<32>;
    } else if #[cfg(target_pointer_width = "64")] {
        const PADDR_BITS: usize = 56;
        pub type Sv39 = Sv<39>;
        pub type Sv48 = Sv<48>;
        pub type Sv57 = Sv<57>;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Sv<const N: usize>;

impl<const N: usize> MmuMeta for Sv<N> {
    const V_ADDR_BITS: usize = N;
    const ADDR_MASK: usize = addr_mask(10, N - OFFSET_BITS);
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

    #[inline]
    fn ppn(value: usize) -> PPN {
        PPN((value >> 10) & PPN_MASK)
    }

    #[inline]
    fn set_ppn(value: &mut usize, ppn: PPN) {
        *value |= (ppn.0 & PPN_MASK) << 10;
    }

    #[inline]
    fn clear_ppn(value: &mut usize) {
        *value &= !(PPN_MASK << 10);
    }
}

impl<const N: usize> Pte<Sv<N>> {}

const PPN_BITS: usize = PADDR_BITS - OFFSET_BITS;
const PPN_MASK: usize = (1 << PPN_BITS) - 1;
const fn addr_mask(base: usize, len: usize) -> usize {
    let m0: usize = !((1 << base) - 1);
    let m1: usize = (1 << (base + len)) - 1;
    (!0) & m0 & m1
}
