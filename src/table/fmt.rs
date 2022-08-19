use super::{PageTableShuttle, Pos, Visitor};
use crate::{Pte, VmMeta, PPN, VPN};
use core::{fmt, marker::PhantomData};

impl<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta>> fmt::Debug for PageTableShuttle<Meta, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.walk(FmtVisitor {
            f,
            max_level: 0,
            new_line: true,
            _phantom: PhantomData,
        });
        Ok(())
    }
}

struct FmtVisitor<'f1, 'f2, Meta: VmMeta> {
    f: &'f1 mut fmt::Formatter<'f2>,
    max_level: usize,
    new_line: bool,
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
        let level = target_hint.level;
        // 如果有效，要打印信息
        if pte.is_valid() {
            if self.new_line {
                for _ in level..self.max_level {
                    for _ in 0..18 {
                        write!(self.f, " ").unwrap();
                    }
                    write!(self.f, " - ").unwrap();
                }
            } else if level < self.max_level {
                // 如果不是第一项，打印一个间隔符
                write!(self.f, " - ").unwrap();
            }
            // 打印页表本身的信息（物理页号）
            write!(self.f, "{:#018x}", pte.ppn().val()).unwrap();
            // 如果是数据页，还要打印映射到的虚页号和自定义的权限信息
            if pte.is_leaf() {
                // 对于大页，打印一些线
                for _ in 0..level {
                    write!(self.f, " - ").unwrap();
                    for _ in 0..18 {
                        write!(self.f, "-").unwrap();
                    }
                }
                // 打印映射的虚址范围和自定义的权限位
                let range = target_hint.vpn.vaddr_range(target_hint.level);
                write!(
                    self.f,
                    " {:#018x}..{:#018x} (",
                    range.start.val(),
                    range.end.val()
                )
                .unwrap();
                Meta::fmt_flags(self.f, pte.flags().0).unwrap();
                write!(self.f, ")").unwrap();
            } else {
                self.new_line = false;
                return target_hint.down();
            }
        }
        // 计算下一个位置
        let vpn = target_hint.next().vpn;
        let next_level = vpn.align_level().min(self.max_level);
        // 如果打印了一个数据页或下一个位置是更高级页，换行
        if pte.is_valid() || (!self.new_line && next_level > level) {
            self.new_line = true;
            writeln!(self.f, "").unwrap();
        }
        Pos::new(vpn, next_level)
    }

    #[inline]
    fn meet(&mut self, _level: usize, _pte: Pte<Meta>, _target_hint: Pos<Meta>) -> Pos<Meta> {
        // 不会跳着遍历
        unreachable!()
    }
}
