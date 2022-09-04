use super::Pos;
use crate::{PageTable, Pte, VmMeta, PPN, VPN};
use core::ptr::NonNull;

/// 页表穿梭机。
///
/// 结合物理页到虚页的翻译算法实现对页表的任意访问。
pub struct PageTableShuttle<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta>> {
    /// 一个页表。
    pub table: PageTable<Meta>,
    /// 翻译函数。
    pub f: F,
}

impl<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta>> PageTableShuttle<Meta, F> {
    /// 使用访问器 `visitor` 遍历页表。
    #[inline]
    pub fn walk(&self, visitor: &mut impl Visitor<Meta>) {
        let mut target = visitor.start(Pos::new(self.table.base, self.table.level));
        walk_inner(&self.table, &self.f, visitor, &mut target);
    }

    /// 使用访问器 `visitor` 遍历并修改页表。
    #[inline]
    pub fn walk_mut(&mut self, visitor: &mut impl Decorator<Meta>) {
        // 先用空的东西把转换函数换出来以规避借用检查
        // FIXME 这能写成 safe 的吗？直接传引用会在递归时产生无限引用。
        use core::mem::{replace, MaybeUninit};
        #[allow(clippy::uninit_assumed_init)]
        let f = replace(&mut self.f, unsafe { MaybeUninit::uninit().assume_init() });
        // 递归遍历，并在结束时把转换函数换回去
        let mut target = visitor.start(Pos::new(self.table.base, self.table.level));
        self.f = walk_inner_mut(&mut self.table, f, visitor, &mut target);
    }
}

/// 递归遍历。
fn walk_inner<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta>>(
    table: &PageTable<Meta>,
    mut f: F,
    visitor: &mut impl Visitor<Meta>,
    target: &mut Pos<Meta>,
) -> F {
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
                let table = unsafe {
                    PageTable::from_raw_parts(
                        NonNull::new_unchecked(f(pte.ppn()).base().as_mut_ptr()),
                        range.start + index * Meta::pages_in_table(level - 1),
                        level - 1,
                    )
                };
                f = walk_inner(&table, f, visitor, target);
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
    f
}

/// 递归遍历。
fn walk_inner_mut<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta>>(
    table: &mut PageTable<Meta>,
    mut f: F,
    visitor: &mut impl Decorator<Meta>,
    target: &mut Pos<Meta>,
) -> F {
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
                let mut table = unsafe {
                    PageTable::from_raw_parts(
                        NonNull::new_unchecked(f(pte.ppn()).base().as_mut_ptr()),
                        range.start + index * Meta::pages_in_table(level - 1),
                        level - 1,
                    )
                };
                f = walk_inner_mut(&mut table, f, visitor, target);
            }
            // 否则请求用户操作
            else {
                match visitor.meet(level, *pte, *target) {
                    // 重设目标
                    Update::Target(new) => *target = new,
                    // 修改页表
                    Update::Pte(new, vpn) => {
                        *pte = new;
                        let mut table = unsafe {
                            PageTable::from_raw_parts(
                                NonNull::new_unchecked(vpn.base().as_mut_ptr()),
                                range.start + index * Meta::pages_in_table(level - 1),
                                level - 1,
                            )
                        };
                        f = walk_inner_mut(&mut table, f, visitor, target);
                    }
                }
            }
        }
        // 访问目标节点
        else {
            *target = visitor.arrive(pte, *target);
        }
    }
    f
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
pub trait Decorator<Meta: VmMeta> {
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
