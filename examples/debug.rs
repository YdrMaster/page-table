use page_table::{MmuMeta, PageTable, PageTableShuttle, VmFlags, VmMeta, PPN, VPN};

fn main() {
    const SUB_FLAGS: VmFlags<Sv39> = unsafe { VmFlags::from_raw(1) };

    #[repr(C, align(4096))]
    struct Page([u8; 1 << Sv39::PAGE_BITS]);

    impl Page {
        #[inline]
        pub const fn new() -> Self {
            Self([0; 1 << Sv39::PAGE_BITS])
        }
    }

    let mut root = Page::new();
    let pt1g0 = Page::new();
    let pt1g7 = Page::new();

    let mut root = unsafe {
        PageTable::<Sv39>::from_raw_parts(root.0.as_mut_ptr().cast(), VPN::ZERO, Sv39::MAX_LEVEL)
    };
    root[0] = SUB_FLAGS.build_pte(PPN::new(pt1g0.0.as_ptr() as usize >> Sv39::PAGE_BITS));
    root[7] = SUB_FLAGS.build_pte(PPN::new(pt1g7.0.as_ptr() as usize >> Sv39::PAGE_BITS));

    println!(
        "{:?}",
        PageTableShuttle {
            table: root,
            f: |ppn| VPN::new(ppn.val())
        }
    )
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Sv39;

impl MmuMeta for Sv39 {
    const P_ADDR_BITS: usize = 56;
    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: &'static [usize] = &[9; 3];
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
