use crate::{mask, MmuMeta, PageTable, PtQuery, Pte, PPN, PT_LEVEL_BITS, VPN};
use core::ops::Range;

pub struct Walker<'a, Meta: MmuMeta, T: Worker<Meta>> {
    worker: &'a mut T,
    root: &'static mut PageTable<Meta>,
    current: Pos,
    target: Pos,
}

#[derive(Clone, Copy)]
pub struct Pos {
    pub vpn: VPN,
    pub level: usize,
}

impl Pos {
    fn range<Meta: MmuMeta>(&self) -> Range<VPN> {
        let start = VPN(self.vpn.0 & !mask(PT_LEVEL_BITS * (self.level + 1)));
        start..start + Meta::pages_in(self.level)
    }
}

pub trait Worker<Meta: MmuMeta> {
    fn walk<'a>(&mut self, vpn: VPN, level: usize, pte: &'a mut Pte<Meta>) -> Option<Pos>;
    fn page_fault<'a>(&mut self, vpn: VPN, level: usize, pte: &'a mut Pte<Meta>) -> bool;
    fn translate(&self, ppn: PPN) -> VPN;
}

impl<'a, Meta: MmuMeta, T: Worker<Meta>> Walker<'a, Meta, T> {
    pub fn new(worker: &'a mut T, root: &'static mut PageTable<Meta>) -> Self {
        Self {
            worker,
            root,
            current: Pos {
                vpn: VPN(0),
                level: Meta::MAX_LEVEL,
            },
            target: Pos {
                vpn: VPN(0),
                level: Meta::MAX_LEVEL,
            },
        }
    }

    pub fn walk(self) {
        self.walk_inner();
    }

    fn walk_inner(mut self) -> Option<Pos> {
        loop {
            if self.target.level > self.current.level {
                return Some(self.target);
            }
            if !self.current.range::<Meta>().contains(&self.target.vpn) {
                return Some(self.target);
            }

            if self.target.level == self.current.level {
                self.current = self.target;
                self.target = self.worker.walk(
                    self.current.vpn,
                    self.current.level,
                    &mut self.root[self.current.vpn.index_in(self.current.level)],
                )?;
            } else {
                match self
                    .root
                    .query_once(self.target.vpn.base(), self.current.level)
                {
                    PtQuery::Invalid => {
                        if !self.worker.page_fault(
                            self.target.vpn,
                            self.current.level,
                            &mut self.root[self.current.vpn.index_in(self.current.level)],
                        ) {
                            return None;
                        }
                    }
                    PtQuery::SubTable(ppn) => {
                        let vpn = self.worker.translate(ppn);
                        let sub = Walker {
                            worker: self.worker,
                            root: unsafe { &mut *vpn.base().as_mut_ptr::<PageTable<Meta>>() },
                            current: Pos {
                                level: self.current.level - 1,
                                ..self.current
                            },
                            target: self.target,
                        };
                        self.target = sub.walk_inner()?;
                    }
                    PtQuery::Leaf(_) => {
                        self.target = self.worker.walk(
                            self.target.vpn,
                            self.current.level,
                            &mut self.root[self.current.vpn.index_in(self.current.level)],
                        )?;
                    }
                }
            }
        }
    }
}
