use std::ops::Add;

use ordered_float::OrderedFloat;

use crate::search::Evaluation;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Eval {
    value: OrderedFloat<f32>,
    spike: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Reward {
    value: OrderedFloat<f32>,
    attack: Option<u32>,
}

impl Eval {
    pub fn new(value: f32) -> Self {
        Eval {
            value: value.into(),
            spike: 0,
        }
    }

    pub fn spike(self) -> u32 {
        self.spike
    }
}

impl Reward {
    pub fn new(value: f32, attack: Option<u32>) -> Self {
        Reward {
            value: value.into(),
            attack,
        }
    }

    pub fn attack(self) -> u32 {
        self.attack.unwrap_or(0)
    }
}

impl Evaluation for Eval {
    type Reward = Reward;

    fn average(of: impl Iterator<Item = Option<Self>>) -> Self {
        let mut count = 0;
        let mut value_sum = 0.0;
        let mut spike_sum = 0;
        for value in of {
            count += 1;
            value_sum += value.map(|e| e.value.0).unwrap_or(-1000.0);
            spike_sum += value.map(|e| e.spike).unwrap_or(0);
        }
        Eval {
            value: (value_sum / count as f32).into(),
            spike: spike_sum / count as u32,
        }
    }
}

impl Add<Reward> for Eval {
    type Output = Self;

    fn add(self, rhs: Reward) -> Eval {
        Eval {
            value: self.value + rhs.value,
            spike: rhs.attack.map_or(0, |attack| self.spike + attack),
        }
    }
}
