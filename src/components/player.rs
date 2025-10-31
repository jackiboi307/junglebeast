use crate::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MoveState {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    hp: u16,
    pub moves: MoveState,
}

impl Player {
    pub fn new() -> Self {
        Self {
            hp: 100,
            moves: MoveState::default(),
        }
    }

    pub fn hp(&self) -> u16 {
        self.hp
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

impl MoveState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_jump(&mut self) {
        self.jump = true;
    }

    pub fn get_jump(&mut self) -> bool {
        let jump = self.jump;
        self.jump = false;
        jump
    }
}
