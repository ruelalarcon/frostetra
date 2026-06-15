use serde::{Deserialize, Serialize};

use crate::bot::behavior::freestyle::Weights;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FreestyleConfig {
    pub weights: Weights,
    pub exploitation: f64,
}
