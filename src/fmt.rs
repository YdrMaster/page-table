use core::marker::PhantomData;

use crate::{
    walker::{Pos, Update, Visitor},
    MmuMeta, PPN, VPN,
};

struct FmtVisitor<Meta: MmuMeta, F: Fn(PPN) -> VPN> {
    f: F,
    _phantom: PhantomData<Meta>,
}

impl<Meta: MmuMeta, F: Fn(PPN) -> VPN> Visitor<Meta> for FmtVisitor<Meta, F> {
    #[inline]
    fn start(&mut self) -> Pos<Meta> {
        // 总是从头开始
        Pos::new(VPN::ZERO, 0)
    }

    #[inline]
    fn translate(&self, ppn: crate::PPN) -> crate::VPN {
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
