use std::ops::Add;

use ordered_float::OrderedFloat;

use crate::search::Evaluation;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Eval {
    value: OrderedFloat<f32>,
}

#[derive(Copy, Clone, Debug)]
pub struct Reward {
    value: OrderedFloat<f32>,
}

impl Eval {
    pub fn new(value: f32) -> Self {
        Eval {
            value: value.into(),
        }
    }
}

impl Reward {
    pub fn new(value: f32) -> Self {
        Reward {
            value: value.into(),
        }
    }
}

impl Evaluation for Eval {
    type Reward = Reward;

    fn average(of: impl Iterator<Item = Option<Self>>) -> Self {
        let mut count = 0;
        let sum: f32 = of
            .map(|v| {
                count += 1;
                v.map(|e| e.value.0).unwrap_or(-1000.0)
            })
            .sum();
        Eval {
            value: (sum / count as f32).into(),
        }
    }
}

impl Add<Reward> for Eval {
    type Output = Self;

    fn add(self, rhs: Reward) -> Eval {
        Eval {
            value: self.value + rhs.value,
        }
    }
}
