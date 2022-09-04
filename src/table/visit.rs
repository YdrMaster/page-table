use super::Pos;
use crate::{PageTable, Pte, VmMeta};
use core::ptr::NonNull;

/// `Meta` 方案的页表访问机制。
pub trait Visitor<Meta: VmMeta> {
    /// 到达 `target_hint` 节点。
    fn arrive(&mut self, pte: Pte<Meta>, target: Pos<Meta>) -> Pos<Meta>;

    /// 在访问 `target` 的过程中，经过一个包括 `target` 的 `level` 级页表项 `pte`，并且这个页表项指向一个中间页表节点。
    fn meet(
        &mut self,
        level: usize,
        pte: Pte<Meta>,
        target: Pos<Meta>,
    ) -> Option<NonNull<Pte<Meta>>>;

    /// 在访问 `target` 的过程中，经过一个包括 `target` 的 `level` 级页表项 `pte`，但这个页表项没有指向一个子页表。
    ///
    /// 以下两种情况会调用这个方法：
    ///
    /// - 访问到包含目标虚页的大页节点；
    /// - 访问到包含目标虚页的无效节点；
    fn block(&mut self, level: usize, pte: Pte<Meta>, target: Pos<Meta>) -> Pos<Meta>;
}

/// `Meta` 方案的页表访问机制。
pub trait Decorator<Meta: VmMeta> {
    /// 到达 `target_hint` 节点。
    fn arrive(&mut self, pte: &mut Pte<Meta>, target_hint: Pos<Meta>) -> Pos<Meta>;

    /// 在访问 `target` 的过程中，经过一个包括 `target` 的 `level` 级页表项 `pte`，并且这个页表项指向一个中间页表节点。
    fn meet(
        &mut self,
        level: usize,
        pte: Pte<Meta>,
        target: Pos<Meta>,
    ) -> Option<NonNull<Pte<Meta>>>;

    /// 在访问 `target` 的过程中，经过一个包括 `target` 的 `level` 级页表项 `pte`。
    ///
    /// 以下两种情况会调用这个方法：
    ///
    /// - 访问到包含目标虚页的大页节点；
    /// - 访问到包含目标虚页的无效节点；
    fn block(&mut self, level: usize, pte: Pte<Meta>, target_hint: Pos<Meta>) -> Update<Meta>;
}

/// 遍历中断时的更新方案。
pub enum Update<Meta: VmMeta> {
    /// 修改目标。
    Target(Pos<Meta>),
    /// 新建中间页表。
    Pte(Pte<Meta>, NonNull<Pte<Meta>>),
}

/// 递归遍历。
pub(super) fn walk_inner<Meta: VmMeta>(
    table: &PageTable<Meta>,
    visitor: &mut impl Visitor<Meta>,
    target: &mut Pos<Meta>,
) {
    let range = table.range();
    let level = table.level;
    // 如果目标虚页不在当前页表覆盖范围内，回到上一级页表
    while level >= target.level && range.contains(&target.vpn) {
        // 计算作为页表项的序号
        let index = target.vpn.index_in(level);
        // 借出页表项
        let pte = table.mem[index];
        // 目标节点等级比当前低需要查页表
        if level > target.level {
            // 有效且不是叶子的页表项是子页表
            if pte.is_valid() && !pte.is_leaf() {
                match visitor.meet(level, pte, *target) {
                    Some(ptr) => {
                        let table = unsafe {
                            PageTable::from_raw_parts(
                                ptr,
                                range.start + index * Meta::pages_in_table(level - 1),
                                level - 1,
                            )
                        };
                        walk_inner(&table, visitor, target);
                    }
                    None => *target = Pos::stop(),
                }
            }
            // 否则请求用户操作
            else {
                *target = visitor.block(level, pte, *target);
            }
        }
        // 访问目标节点
        else {
            *target = visitor.arrive(pte, *target);
        }
    }
}

/// 递归遍历。
pub(super) fn walk_inner_mut<Meta: VmMeta>(
    table: &mut PageTable<Meta>,
    visitor: &mut impl Decorator<Meta>,
    target: &mut Pos<Meta>,
) {
    let range = table.range();
    let level = table.level;
    // 如果目标虚页不在当前页表覆盖范围内，回到上一级页表
    while level >= target.level && range.contains(&target.vpn) {
        // 计算作为页表项的序号
        let index = target.vpn.index_in(level);
        // 借出页表项
        let pte = &mut table.mem[index];
        // 目标节点等级比当前低需要查页表
        if level > target.level {
            // 有效且不是叶子的页表项是子页表
            if pte.is_valid() && !pte.is_leaf() {
                match visitor.meet(level, *pte, *target) {
                    Some(ptr) => {
                        let mut table = unsafe {
                            PageTable::from_raw_parts(
                                ptr,
                                range.start + index * Meta::pages_in_table(level - 1),
                                level - 1,
                            )
                        };
                        walk_inner_mut(&mut table, visitor, target);
                    }
                    None => *target = Pos::stop(),
                }
            }
            // 否则请求用户操作
            else {
                match visitor.block(level, *pte, *target) {
                    // 重设目标
                    Update::Target(new) => *target = new,
                    // 修改页表
                    Update::Pte(new, ptr) => {
                        *pte = new;
                        let mut table = unsafe {
                            PageTable::from_raw_parts(
                                ptr,
                                range.start + index * Meta::pages_in_table(level - 1),
                                level - 1,
                            )
                        };
                        walk_inner_mut(&mut table, visitor, target);
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
