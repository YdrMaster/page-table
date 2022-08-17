use super::Pos;
use crate::{PageTable, Pte, VmMeta, PPN, VPN};

/// 页表穿梭机。
///
/// 结合物理页到虚页的翻译算法实现对页表的任意访问。
pub struct PageTableShuttle<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta>> {
    /// 一个页表。
    pub table: PageTable<Meta>,
    /// 翻译函数。
    pub f: F,
}

impl<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta> + Clone> PageTableShuttle<Meta, F> {
    /// 使用访问器 `visitor` 遍历页表。
    #[inline]
    pub fn walk(&self, mut visitor: impl Visitor<Meta>) {
        let mut target = visitor.start(Pos::new(self.table.base, self.table.level));
        self.walk_inner(&mut visitor, &mut target);
    }

    /// 使用访问器 `visitor` 遍历并修改页表。
    #[inline]
    pub fn walk_mut(&mut self, mut visitor: impl VisitorMut<Meta>) {
        let mut target = visitor.start(Pos::new(self.table.base, self.table.level));
        self.walk_inner_mut(&mut visitor, &mut target);
    }

    /// 递归遍历。
    fn walk_inner(&self, visitor: &mut impl Visitor<Meta>, target: &mut Pos<Meta>) {
        let range = self.table.range();
        let level = self.table.level;
        // 如果目标虚页不在当前页表覆盖范围内，回到上一级页表
        while level >= target.level && range.contains(&target.vpn) {
            // 计算作为页表项的序号
            let index = target.vpn.index_in(level);
            // 借出页表项
            let pte = self.table.mem[index];
            // 目标节点等级比当前低需要查页表
            if level > target.level {
                // 有效且不是叶子的页表项是子页表
                if pte.is_valid() && !pte.is_leaf() {
                    PageTableShuttle {
                        table: unsafe {
                            PageTable::from_raw_parts(
                                (self.f)(pte.ppn()).base().as_mut_ptr(),
                                range.start + index * Meta::pages_in_table(level - 1),
                                level - 1,
                            )
                        },
                        f: self.f.clone(),
                    }
                    .walk_inner(visitor, target);
                }
                // 否则请求用户操作
                else {
                    *target = visitor.meet(level, pte, *target);
                }
            }
            // 访问目标节点
            else {
                *target = visitor.arrive(pte, *target);
            }
        }
    }

    /// 递归遍历。
    fn walk_inner_mut(&mut self, visitor: &mut impl VisitorMut<Meta>, target: &mut Pos<Meta>) {
        let range = self.table.range();
        let level = self.table.level;
        // 如果目标虚页不在当前页表覆盖范围内，回到上一级页表
        while level >= target.level && range.contains(&target.vpn) {
            // 计算作为页表项的序号
            let index = target.vpn.index_in(level);
            // 借出页表项
            let pte = &mut self.table.mem[index];
            // 目标节点等级比当前低需要查页表
            if level > target.level {
                // 有效且不是叶子的页表项是子页表
                if pte.is_valid() && !pte.is_leaf() {
                    PageTableShuttle {
                        table: unsafe {
                            PageTable::from_raw_parts(
                                (self.f)(pte.ppn()).base().as_mut_ptr(),
                                range.start + index * Meta::pages_in_table(level - 1),
                                level - 1,
                            )
                        },
                        f: &self.f,
                    }
                    .walk_inner_mut(visitor, target);
                }
                // 否则请求用户操作
                else {
                    match visitor.meet(level, *pte, *target) {
                        // 重设目标
                        Update::Target(new) => *target = new,
                        // 修改页表
                        Update::Pte(new, vpn) => {
                            *pte = new;
                            PageTableShuttle {
                                table: unsafe {
                                    PageTable::from_raw_parts(
                                        vpn.base().as_mut_ptr(),
                                        range.start + index * Meta::pages_in_table(level - 1),
                                        level - 1,
                                    )
                                },
                                f: &self.f,
                            }
                            .walk_inner_mut(visitor, target);
                        }
                    }
                }
            }
            // 访问目标节点
            else {
                *target = visitor.arrive(pte, *target);
            }
        }
    }
}

/// `Meta` 方案的页表访问机制。
pub trait Visitor<Meta: VmMeta> {
    /// 出发时调用一次以设置第一个目标。
    ///
    /// `pos` 是页表上最高级别的第一个页的位置。
    fn start(&mut self, pos: Pos<Meta>) -> Pos<Meta>;

    /// 到达 `target_hint` 节点。
    fn arrive(&mut self, pte: Pte<Meta>, target_hint: Pos<Meta>) -> Pos<Meta>;

    /// 在访问 `target` 的过程中，经过一个包括 `target` 的 `level` 级页表项 `pte`。
    ///
    /// 以下两种情况会调用这个方法：
    ///
    /// - 访问到包含目标虚页的大页节点；
    /// - 访问到包含目标虚页的无效节点；
    fn meet(&mut self, level: usize, pte: Pte<Meta>, target_hint: Pos<Meta>) -> Pos<Meta>;
}

/// `Meta` 方案的页表访问机制。
pub trait VisitorMut<Meta: VmMeta> {
    /// 出发时调用一次以设置第一个目标。
    ///
    /// `pos` 是页表上最高级别的第一个页的位置。
    fn start(&mut self, pos: Pos<Meta>) -> Pos<Meta>;

    /// 到达 `target_hint` 节点。
    fn arrive(&mut self, pte: &mut Pte<Meta>, target_hint: Pos<Meta>) -> Pos<Meta>;

    /// 在访问 `target` 的过程中，经过一个包括 `target` 的 `level` 级页表项 `pte`。
    ///
    /// 以下两种情况会调用这个方法：
    ///
    /// - 访问到包含目标虚页的大页节点；
    /// - 访问到包含目标虚页的无效节点；
    fn meet(&mut self, level: usize, pte: Pte<Meta>, target_hint: Pos<Meta>) -> Update<Meta>;
}

/// 遍历中断时的更新方案。
pub enum Update<Meta: VmMeta> {
    /// 修改目标。
    Target(Pos<Meta>),
    /// 新建中间页表。
    Pte(Pte<Meta>, VPN<Meta>),
}
