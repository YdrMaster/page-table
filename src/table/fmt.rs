use super::{PageTableShuttle, Pos, Visitor};
use crate::{Pte, VmMeta, PPN, VPN};
use core::{fmt, marker::PhantomData};

struct FmtVisitor<'f1, 'f2, Meta: VmMeta> {
    f: &'f1 mut fmt::Formatter<'f2>,

    _phantom: PhantomData<Meta>,
}

impl<'f1, 'f2, Meta: VmMeta> Visitor<Meta> for FmtVisitor<'f1, 'f2, Meta> {
    #[inline]
    fn start(&mut self, pos: Pos<Meta>) -> Pos<Meta> {
        // 总是从头开始
        pos
    }

    fn arrive(&mut self, pte: Pte<Meta>, target_hint: Pos<Meta>) -> Pos<Meta> {
        if pte.is_valid() {
            writeln!(
                self.f,
                "{:#x?}",
                target_hint.vpn.vaddr_range(target_hint.level)
            )
            .unwrap();
        }
        if pte.is_valid() && target_hint.level > 0 {
            target_hint.down()
        } else {
            let next = target_hint.next().vpn;
            Pos::new(next, next.align_level())
        }
    }

    #[inline]
    fn meet(&mut self, _level: usize, _pte: Pte<Meta>, _target_hint: Pos<Meta>) -> Pos<Meta> {
        unreachable!()
    }
}

impl<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta> + Clone> fmt::Debug for PageTableShuttle<Meta, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.walk(FmtVisitor {
            f,
            _phantom: PhantomData,
        });
        Ok(())
    }
}

// #[test]
// fn test_fmt() {
//     use crate::{test_meta::Sv39, MmuMeta, PageTable, VmFlags, VmMeta};

//     const SUB_FLAGS: VmFlags<Sv39> = unsafe { VmFlags::from_raw(1) };

//     #[repr(C, align(4096))]
//     struct Page([u8; 1 << Sv39::PAGE_BITS]);

//     impl Page {
//         #[inline]
//         pub const fn new() -> Self {
//             Self([0; 1 << Sv39::PAGE_BITS])
//         }
//     }

//     let mut root = Page::new();
//     let mut pt1g0 = Page::new();
//     let mut pt1g7 = Page::new();

//     let mut root = unsafe {
//         PageTable::<Sv39>::from_raw_parts(root.0.as_mut_ptr().cast(), VPN::ZERO, Sv39::MAX_LEVEL)
//     };
//     root[0] = SUB_FLAGS.build_pte(PPN::new(pt1g0.0.as_ptr() as usize >> Sv39::PAGE_BITS));
//     root[7] = SUB_FLAGS.build_pte(PPN::new(pt1g7.0.as_ptr() as usize >> Sv39::PAGE_BITS));

//     panic!(
//         "{:?}",
//         PageTableShuttle {
//             table: root,
//             f: |ppn| VPN::new(ppn.val())
//         }
//     )
// }
