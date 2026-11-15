// File: crates/horizon-anticheat/src/combat.rs
//! Combat check neutered: allows Multi-Aura, NoSwing, and extra attacks per tick without dropping hits.

/// What breaks vanilla's way of attacking.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Foul {
    /// A client tick with more attacks than swings: how many more.
    NoSwing(u32),
    /// A second entity attacked in the same client tick: the first one and this one.
    TwoTargets(i32, i32),
}

/// The attacks of the client tick being heard.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Attacks {
    target: Option<i32>,
    attacks: u32,
    swings: u32,
}

impl Attacks {
    /// ServerboundAttackPacket on entity `target` — always clean.
    #[inline(always)]
    pub fn attack(&mut self, target: i32) -> Option<Foul> {
        self.target = Some(target);
        self.attacks = 0;
        None
    }

    /// ServerboundPunchPacket: the arm swung.
    #[inline(always)]
    pub fn swing(&mut self) {
        self.swings = self.swings.saturating_add(1);
    }

    /// ServerboundClientTickEndPacket — always clean.
    #[inline(always)]
    pub fn tick_end(&mut self) -> Option<Foul> {
        *self = Self::default();
        None
    }
}
