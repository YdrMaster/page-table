use page_table::{MmuMeta, PageTable, PageTableFormatter, VmFlags, VmMeta, PPN, VPN};
use std::ptr::NonNull;

/// 按 Sv39 的方案修饰任意架构的用户态指针有概率出问题。需要重写测例，支持更好的虚拟化。
fn main() {
    const SUB_FLAGS: VmFlags<Sv39> = unsafe { VmFlags::from_raw(1) };
    const XRP_FLAGS: VmFlags<Sv39> = unsafe { VmFlags::from_raw(0b1011) };
    const ROP_FLAGS: VmFlags<Sv39> = unsafe { VmFlags::from_raw(0b0011) };

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
    let pt1g9 = Page::new();
    let mut pt1g7 = Page::new();
    let mut pt2m4 = Page::new();

    let mut root = unsafe {
        PageTable::<Sv39>::from_raw_parts(
            NonNull::new_unchecked(root.0.as_mut_ptr().cast()),
            VPN::ZERO,
            Sv39::MAX_LEVEL,
        )
    };
    root[0] = SUB_FLAGS.build_pte(PPN::new(pt1g0.0.as_ptr() as usize >> Sv39::PAGE_BITS));
    root[7] = SUB_FLAGS.build_pte(PPN::new(pt1g7.0.as_ptr() as usize >> Sv39::PAGE_BITS));
    root[9] = SUB_FLAGS.build_pte(PPN::new(pt1g9.0.as_ptr() as usize >> Sv39::PAGE_BITS));

    let mut pt1g7 = unsafe {
        PageTable::<Sv39>::from_raw_parts(
            NonNull::new_unchecked(pt1g7.0.as_mut_ptr().cast()),
            VPN::ZERO,
            Sv39::MAX_LEVEL,
        )
    };
    pt1g7[0] = ROP_FLAGS.build_pte(PPN::new(0x12345678));
    pt1g7[4] = SUB_FLAGS.build_pte(PPN::new(pt2m4.0.as_ptr() as usize >> Sv39::PAGE_BITS));

    let mut pt2m4 = unsafe {
        PageTable::<Sv39>::from_raw_parts(
            NonNull::new_unchecked(pt2m4.0.as_mut_ptr().cast()),
            VPN::ZERO,
            Sv39::MAX_LEVEL,
        )
    };
    for i in 12..18 {
        pt2m4[i] = XRP_FLAGS.build_pte(PPN::new(0x23300 + i as usize));
    }
    for i in 31..40 {
        pt2m4[i] = ROP_FLAGS.build_pte(PPN::new(0x23300 + i as usize));
    }

    println!(
        "{:?}",
        PageTableFormatter {
            pt: root,
            f: |ppn| unsafe {
                NonNull::new_unchecked(VPN::<Sv39>::new(ppn.val()).base().as_mut_ptr())
            }
        }
    );
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct Sv39;

impl MmuMeta for Sv39 {
    const P_ADDR_BITS: usize = 56;
    const PAGE_BITS: usize = 12;
    const LEVEL_BITS: &'static [usize] = &[9; 3];
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
