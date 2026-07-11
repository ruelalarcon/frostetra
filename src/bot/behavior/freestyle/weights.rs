use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Weights {
    pub bumpiness: f32,
    pub bumpiness_sq: f32,
    pub row_transitions: f32,
    pub height: f32,
    pub top_half: f32,
    pub top_quarter: f32,
    pub jeopardy: f32,
    pub cavity_cells: f32,
    pub cavity_cells_sq: f32,
    pub overhang_cells: f32,
    pub overhang_cells_sq: f32,
    pub covered_cells: f32,
    pub covered_cells_sq: f32,
    pub tslot: [f32; 4],
    pub well_depth: f32,
    pub max_well_depth: u32,
    pub well_column: [f32; 10],

    pub has_back_to_back: f32,
    pub back_to_back_depth: f32,
    pub wasted_t: f32,
    pub move_time: f32,

    pub normal_clears: [f32; 5],
    pub t_spin_clears: [f32; 4],
    pub t_spin_mini_clears: [f32; 4],
    pub allspin_clears: [f32; 4],
    pub back_to_back_clear: f32,
    pub combo_garbage: f32,
    pub perfect_clear: f32,
    pub use_bag: bool,
    pub timed_jeopardy: bool,
    pub stack_pc_damage: bool,
}
