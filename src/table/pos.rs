use crate::{VmMeta, VPN};
use core::fmt;

/// `Meta` 方案中页表上的一个位置。
#[derive(Clone, Copy)]
pub struct Pos<Meta: VmMeta> {
    /// 目标页表项包含的一个虚页号。
    pub vpn: VPN<Meta>,
    /// 目标页表项的级别。
    pub level: usize,
}

impl<Meta: VmMeta> Pos<Meta> {
    /// 新建目标。
    #[inline]
    pub const fn new(vpn: VPN<Meta>, level: usize) -> Self {
        Self { vpn, level }
    }

    /// 结束遍历。
    #[inline]
    pub const fn stop() -> Self {
        Self {
            vpn: VPN::ZERO,
            level: Meta::MAX_LEVEL + 1,
        }
    }

    /// 向前移动一页。
    #[inline]
    pub fn prev(self) -> Self {
        let delta = match self.level {
            0 => 1,
            n => Meta::pages_in_table(n - 1),
        };
        match self.vpn.val().checked_sub(delta) {
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
        let delta = match self.level {
            0 => 1,
            n => Meta::pages_in_table(n - 1),
        };
        match self.vpn.val().checked_add(delta) {
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

impl<Meta: VmMeta> fmt::Debug for Pos<Meta> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Pos {{ vpn = {:#x}, level = {:#x} }}",
            self.vpn.val(),
            self.level
        )
    }
}
