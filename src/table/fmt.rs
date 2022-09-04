use super::{Pos, Visitor};
use crate::{PageTable, Pte, VmMeta, PPN};
use core::{fmt, marker::PhantomData, ptr::NonNull};

/// 页表格式化器。
///
/// 为了遍历，需要知道在当前地址空间访问物理页的方法。
pub struct PageTableFormatter<Meta: VmMeta, F: Fn(PPN<Meta>) -> NonNull<Pte<Meta>>> {
    /// 根页表。
    pub pt: PageTable<Meta>,
    /// 物理页转换为指针。
    pub f: F,
}

impl<Meta: VmMeta, F: Fn(PPN<Meta>) -> NonNull<Pte<Meta>>> fmt::Debug
    for PageTableFormatter<Meta, F>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pt.walk(
            Pos {
                vpn: self.pt.base,
                level: 0,
            },
            &mut FmtVisitor {
                f,
                t: &self.f,
                max_level: self.pt.level,
                level: self.pt.level + 1,
                _phantom: PhantomData,
            },
        );
        Ok(())
    }
}

struct FmtVisitor<'f1, 'f2, Meta: VmMeta, T: Fn(PPN<Meta>) -> NonNull<Pte<Meta>>> {
    f: &'f1 mut fmt::Formatter<'f2>,
    t: T,
    max_level: usize,
    level: usize,
    _phantom: PhantomData<Meta>,
}

impl<'f1, 'f2, Meta: VmMeta, T: Fn(PPN<Meta>) -> NonNull<Pte<Meta>>> FmtVisitor<'f1, 'f2, Meta, T> {
    /// 打印一个物理页号。
    fn ppn(&mut self, ppn: PPN<Meta>, level: usize) {
        if level >= self.level {
            writeln!(self.f).unwrap();
            for _ in level..self.max_level {
                for _ in 0..18 {
                    write!(self.f, " ").unwrap();
                }
                write!(self.f, " - ").unwrap();
            }
        }
        write!(self.f, "{:#018x}", ppn.val()).unwrap();
    }

    /// 打印一个页表项。
    fn pte(&mut self, pte: Pte<Meta>, pos: Pos<Meta>) {
        // 打印映射的虚址范围和自定义的权限位
        let range = pos.vpn.vaddr_range(pos.level);
        write!(
            self.f,
            " {:#018x}..{:#018x} (",
            range.start.val(),
            range.end.val()
        )
        .unwrap();
        Meta::fmt_flags(self.f, pte.flags().val()).unwrap();
        write!(self.f, ")").unwrap();
        self.level = pos.level;
    }
}

impl<'f1, 'f2, Meta: VmMeta, T: Fn(PPN<Meta>) -> NonNull<Pte<Meta>>> Visitor<Meta>
    for FmtVisitor<'f1, 'f2, Meta, T>
{
    #[inline]
    fn meet(
        &mut self,
        level: usize,
        pte: Pte<Meta>,
        _target: Pos<Meta>,
    ) -> Option<core::ptr::NonNull<Pte<Meta>>> {
        self.ppn(pte.ppn(), level);
        write!(self.f, " - ").unwrap();
        self.level = level;
        Some((self.t)(pte.ppn()))
    }

    #[inline]
    fn block(&mut self, level: usize, pte: Pte<Meta>, target: Pos<Meta>) -> Pos<Meta> {
        if pte.is_valid() {
            self.ppn(pte.ppn(), level);
            // 打印一些横线
            for _ in 0..level {
                write!(self.f, " - ").unwrap();
                for _ in 0..18 {
                    write!(self.f, "-").unwrap();
                }
            }
            // 打印映射的虚址范围和自定义的权限位
            self.pte(pte, Pos { level, ..target });
        }
        Pos {
            level: 0,
            ..Pos::new(target.vpn, level).next()
        }
    }

    fn arrive(&mut self, pte: Pte<Meta>, target: Pos<Meta>) -> Pos<Meta> {
        // 如果有效，要打印信息
        if pte.is_valid() {
            self.ppn(pte.ppn(), 0);
            self.pte(pte, target);
        }
        target.next()
    }
}
