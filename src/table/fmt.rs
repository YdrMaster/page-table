use super::{PageTableShuttle, Pos, Visitor};
use crate::{Pte, VmMeta, PPN, VPN};
use core::{fmt, marker::PhantomData};

struct FmtVisitor<'f1, 'f2, Meta: VmMeta> {
    f: &'f1 mut fmt::Formatter<'f2>,
    max_level: usize,
    last_level: Option<usize>,
    _phantom: PhantomData<Meta>,
}

impl<'f1, 'f2, Meta: VmMeta> Visitor<Meta> for FmtVisitor<'f1, 'f2, Meta> {
    #[inline]
    fn start(&mut self, pos: Pos<Meta>) -> Pos<Meta> {
        self.max_level = pos.level;
        // 总是从头开始
        pos
    }

    fn arrive(&mut self, pte: Pte<Meta>, target_hint: Pos<Meta>) -> Pos<Meta> {
        if pte.is_valid() {
            write!(self.f, ". ").unwrap();
            if pte.is_leaf() {
            } else {
                self.last_level = Some(target_hint.level);
                return target_hint.down();
            }
        }

        let vpn = target_hint.next().vpn;
        let level = vpn.align_level().min(self.max_level);
        if let Some(lv) = self.last_level.take() {
            if level >= lv {
                writeln!(self.f, "").unwrap();
            } else {
                self.last_level = Some(lv);
            }
        }

        Pos::new(vpn, level)
    }

    #[inline]
    fn meet(&mut self, _level: usize, _pte: Pte<Meta>, _target_hint: Pos<Meta>) -> Pos<Meta> {
        // 不会跳着遍历
        unreachable!()
    }
}

impl<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta> + Clone> fmt::Debug for PageTableShuttle<Meta, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.walk(FmtVisitor {
            f,
            max_level: 0,
            last_level: None,
            _phantom: PhantomData,
        });
        Ok(())
    }
}
