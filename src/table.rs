﻿use crate::{Pte, VmMeta, PPN, VPN};
use core::{
    marker::PhantomData,
    ops::{Index, IndexMut, Range},
};

/// 页表。
///
/// 不持有页表的所有权，因为页表总是在一些物理页帧上。
pub struct PageTable<Meta: VmMeta> {
    mem: &'static mut [Pte<Meta>],
    base: VPN<Meta>,
    level: usize,
}

impl<Meta: VmMeta> PageTable<Meta> {
    /// 从指向第一个页表项的指针创建页表。
    ///
    /// # Safety
    ///
    /// 同 [from_raw_parts_mut](core::slice::from_raw_parts_mut).
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut Pte<Meta>, base: VPN<Meta>, level: usize) -> Self {
        Self {
            mem: core::slice::from_raw_parts_mut(ptr, 1 << Meta::LEVEL_BITS[level]),
            base: base.floor(level),
            level,
        }
    }

    /// 获取指向第一个页表项的指针。
    #[inline]
    pub const fn as_ptr(&self) -> *const Pte<Meta> {
        self.mem.as_ptr()
    }

    /// 获取页表级别。
    #[inline]
    pub const fn level(&self) -> usize {
        self.level
    }

    /// 获取页表容纳的虚页号范围。
    #[inline]
    pub fn range(&self) -> Range<VPN<Meta>> {
        self.base..self.base + Meta::pages_in_table(self.level)
    }

    /// 使用访问器 `visitor` 遍历页表。
    #[inline]
    pub fn walk(&mut self, mut visitor: impl Visitor<Meta>) {
        let mut target = visitor.start(Pos::new(self.base, self.level));
        self.walk_inner(&mut visitor, &mut target);
    }

    /// 递归遍历。
    fn walk_inner(&mut self, visitor: &mut impl Visitor<Meta>, target: &mut Pos<Meta>) {
        let range = self.range();
        let level = self.level;
        // 如果目标虚页不在当前页表覆盖范围内，回到上一级页表
        while level >= target.level && range.contains(&target.vpn) {
            // 计算作为页表项的序号
            let index = target.vpn.index_in(level);
            // 借出页表项
            let pte = &mut self.mem[index];
            // 目标节点等级比当前低需要查页表
            if level > target.level {
                // 有效且不是叶子的页表项是子页表
                if pte.is_valid() && !pte.is_leaf() {
                    unsafe {
                        PageTable::from_raw_parts(
                            visitor.translate(pte.ppn()).base().as_mut_ptr(),
                            range.start + index * Meta::pages_in_table(level - 1),
                            level - 1,
                        )
                    }
                    .walk_inner(visitor, target);
                }
                // 否则请求用户操作
                else {
                    match visitor.meet(level, *pte, *target) {
                        // 重设目标
                        Update::Target(new) => *target = new,
                        // 修改页表
                        Update::Pte(new, vpn) => {
                            *pte = new;
                            unsafe {
                                PageTable::from_raw_parts(
                                    vpn.base().as_mut_ptr(),
                                    range.start + index * Meta::pages_in_table(level - 1),
                                    level - 1,
                                )
                            }
                            .walk_inner(visitor, target);
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

impl<Meta: VmMeta> Index<usize> for PageTable<Meta> {
    type Output = Pte<Meta>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.mem[index]
    }
}

impl<Meta: VmMeta> IndexMut<usize> for PageTable<Meta> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.mem[index]
    }
}

/// `Meta` 方案的页表访问机制。
pub trait Visitor<Meta: VmMeta> {
    /// 出发时调用一次以设置第一个目标。
    ///
    /// `pos` 是页表上最高级别的第一个页的位置。
    fn start(&mut self, pos: Pos<Meta>) -> Pos<Meta>;

    /// 在访问目标节点的过程中，经过一个位于 `ppn` 物理页中间页表，需要计算这个物理页的虚页号。
    fn translate(&self, ppn: PPN<Meta>) -> VPN<Meta>;

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

/// `Meta` 方案中页表上的一个位置。
#[derive(Clone, Copy)]
pub struct Pos<Meta: VmMeta> {
    /// 目标页表项包含的一个虚页号。
    pub vpn: VPN<Meta>,
    /// 目标页表项的级别。
    pub level: usize,
    _phantom: PhantomData<Meta>,
}

impl<Meta: VmMeta> Pos<Meta> {
    /// 新建目标。
    #[inline]
    pub const fn new(vpn: VPN<Meta>, level: usize) -> Self {
        Self {
            vpn,
            level,
            _phantom: PhantomData,
        }
    }

    /// 结束遍历。
    #[inline]
    pub const fn stop() -> Self {
        Self {
            vpn: VPN::ZERO,
            level: usize::MAX,
            _phantom: PhantomData,
        }
    }

    /// 向前移动一页。
    #[inline]
    pub fn prev(self) -> Self {
        match self.vpn.val().checked_sub(Meta::pages_in_table(self.level)) {
            Some(vpn) => Self {
                vpn: VPN::new(vpn),
                ..self
            },
            None => panic!("prev: vpn overflow"),
        }
    }

    /// 向后移动一页。
    #[inline]
    pub fn next(self) -> Self {
        match self.vpn.val().checked_add(Meta::pages_in_table(self.level)) {
            Some(vpn) => Self {
                vpn: VPN::new(vpn),
                ..self
            },
            None => panic!("next: vpn overflow"),
        }
    }

    /// 向上移动一页。
    #[inline]
    pub fn up(self) -> Self {
        match self.level.checked_add(1) {
            Some(level) => Self { level, ..self },
            None => panic!("up: level overflow"),
        }
    }

    /// 向下移动一页。
    #[inline]
    pub fn down(self) -> Self {
        match self.level.checked_sub(1) {
            Some(level) => Self { level, ..self },
            None => panic!("down: level overflow"),
        }
    }
}

/// 遍历中断时的更新方案。
pub enum Update<Meta: VmMeta> {
    /// 修改目标。
    Target(Pos<Meta>),
    /// 新建中间页表。
    Pte(Pte<Meta>, VPN<Meta>),
}
