use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    hp: u16,
}

impl Player {
    pub fn new() -> Self {
        Self {
            hp: 100,
        }
    }

    pub fn hurt(&mut self, damage: u16) {
        self.hp = self.hp.saturating_sub(damage);
    }

    pub fn dead(&self) -> bool {
        self.hp == 0
    }

    pub fn reset_hp(&mut self) {
        self.hp = 100;
    }
}
