use crate::{
    visit::{Pos, Update, Visitor},
    VmMeta, PPN, VPN,
};
use core::marker::PhantomData;

struct FmtVisitor<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta>> {
    f: F,
    _phantom: PhantomData<Meta>,
}

impl<Meta: VmMeta, F: Fn(PPN<Meta>) -> VPN<Meta>> Visitor<Meta> for FmtVisitor<Meta, F> {
    #[inline]
    fn start(&mut self, level: usize) -> Pos<Meta> {
        // 总是从头开始
        Pos::new(VPN::ZERO, level)
    }

    #[inline]
    fn translate(&self, ppn: PPN<Meta>) -> VPN<Meta> {
        (self.f)(ppn)
    }

    fn arrive(&mut self, pte: &mut crate::Pte<Meta>, target_hint: Pos<Meta>) -> Pos<Meta> {
        todo!()
    }

    fn meet(
        &mut self,
        level: usize,
        pte: crate::Pte<Meta>,
        target_hint: Pos<Meta>,
    ) -> Update<Meta> {
        todo!()
    }
}
