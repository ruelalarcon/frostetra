use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Weights {
    pub cell_coveredness: f32,
    pub max_cell_covered_height: u32,
    pub holes: f32,
    pub row_transitions: f32,
    pub height: f32,
    pub height_upper_half: f32,
    pub height_upper_quarter: f32,
    pub garbage_pressure_activation_height: u32,
    pub garbage_pressure_rows: u32,
    pub garbage_pressure_weight: f32,
    pub tetris_well_depth: f32,
    pub tslot: [f32; 4],

    pub has_back_to_back: f32,
    pub back_to_back_depth: f32,
    pub wasted_t: f32,
    pub softdrop: f32,

    pub normal_clears: [f32; 5],
    pub t_spin_clears: [f32; 4],
    pub t_spin_mini_clears: [f32; 4],
    pub allspin_clears: [f32; 4],
    pub back_to_back_clear: f32,
    pub combo_attack: f32,
    pub perfect_clear: f32,
    pub perfect_clear_override: bool,
}
