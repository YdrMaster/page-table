//! 遍历页表。

use crate::{MmuMeta, PageTable, Pte, PPN, VPN};
use core::marker::PhantomData;

/// `Meta` 方案的页表访问机制。
pub trait Visitor<Meta: MmuMeta> {
    /// 在访问 `target` 的过程中，访问一个 `level` 级页表项 `pte`。
    ///
    /// 以下 3 种情况会调用这个方法：
    ///
    /// - 访问到目标页表项；
    /// - 访问到包含目标虚页的大页节点；
    /// - 访问到包含目标虚页的无效节点；
    fn walk(&mut self, level: usize, target: &mut Pos<Meta>, pte: &mut Pte<Meta>);

    /// 将一个物理页号转换为当前地址空间的虚页号。
    ///
    /// 当访问目标虚页的过程中遇到中间页表时调用这个方法。
    fn translate(&self, ppn: PPN) -> VPN;
}

/// `Meta` 方案中页表上的一个位置。
pub struct Pos<Meta: MmuMeta> {
    /// 目标页表项包含的一个虚页号。
    pub vpn: VPN,
    /// 目标页表项的级别。
    pub level: usize,
    _phantom: PhantomData<Meta>,
}

impl<Meta: MmuMeta> Pos<Meta> {
    /// 根页表的位置。
    pub const ROOT: Self = Self {
        vpn: VPN(0),
        level: Meta::MAX_LEVEL,
        _phantom: PhantomData,
    };

    /// 向前移动一页。
    #[inline]
    pub fn prev(&mut self) {
        match self.vpn.0.checked_sub(Meta::pages_in(self.level)) {
            Some(vpn) => self.vpn.0 = vpn,
            None => panic!("prev: vpn overflow"),
        }
    }

    /// 向后移动一页。
    #[inline]
    pub fn next(&mut self) {
        match self.vpn.0.checked_add(Meta::pages_in(self.level)) {
            Some(vpn) => self.vpn.0 = vpn,
            None => panic!("next: vpn overflow"),
        }
    }

    /// 向上移动一页。
    #[inline]
    pub fn up(&mut self) {
        match self.level.checked_add(1) {
            Some(level) => self.level = level,
            None => panic!("up: level overflow"),
        }
    }

    /// 向下移动一页。
    #[inline]
    pub fn down(&mut self) {
        match self.level.checked_sub(1) {
            Some(level) => self.level = level,
            None => panic!("down: level overflow"),
        }
    }

    /// 结束遍历。
    #[inline]
    pub fn stop(&mut self) {
        self.level = usize::MAX;
    }
}

/// 使用访问器 `visitor` 遍历虚址空间 `root`。
pub fn walk<Meta: MmuMeta>(mut visitor: impl Visitor<Meta>, root: &mut PageTable<Meta>) {
    let mut target = Pos::ROOT;
    walk_inner(&mut visitor, root, &mut target, VPN(0), Meta::MAX_LEVEL);
}

/// 递归遍历。
fn walk_inner<Meta: MmuMeta>(
    visitor: &mut impl Visitor<Meta>,
    table: &mut PageTable<Meta>,
    target: &mut Pos<Meta>,
    base: VPN,
    level: usize,
) {
    // 如果目标虚页不在当前页表覆盖范围内，回到上一级页表
    while level <= target.level && (base..base + Meta::pages_in(level)).contains(&target.vpn) {
        // 计算作为页表项的序号
        let index = target.vpn.index_in(level);
        // 借出页表项
        let pte = &mut table[index];
        // 如果目标节点等级比当前低需要查页表
        // 如果有效且不是叶子的页表项是子页表
        if target.level < level && pte.is_valid() && !pte.is_leaf() {
            let table = unsafe {
                &mut *visitor
                    .translate(pte.ppn())
                    .base()
                    .as_mut_ptr::<PageTable<Meta>>()
            };
            let level = level - 1;
            let base = base + index * Meta::pages_in(level);
            walk_inner(visitor, table, target, base, level);
        }
        // 访问目标节点、无效节点或叶子节点
        else {
            visitor.walk(level, target, pte);
        }
    }
}
