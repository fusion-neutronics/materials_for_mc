use serde::{Deserialize, Serialize};
use crate::nuclide::Nuclide;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub cross_section: Vec<f64>,
    pub threshold_idx: usize,
    pub interpolation: Vec<i32>,
    #[serde(skip, default)]
    pub energy: Vec<f64>,  // Reaction-specific energy grid
    pub mt_number: i32,  // The MT number for this reaction
}

/// Populates the energy grid for all reactions in a nuclide
/// 
/// Each reaction has a threshold_idx which indicates the starting index
/// in the nuclide's energy grid where the reaction begins.
/// This function populates the energy field of each reaction with the
/// appropriate slice of the nuclide's energy grid.
pub fn populate_reaction_energy_grids(nuclide: &mut Nuclide) {
    if let Some(energy_map) = &nuclide.energy {
        for (temp, temp_reactions) in nuclide.reactions.iter_mut() {
            if let Some(energy_grid) = energy_map.get(temp) {
                for (_, reaction) in temp_reactions.iter_mut() {
                    let threshold_idx = reaction.threshold_idx;
                    if threshold_idx < energy_grid.len() {
                        // Slice the energy grid from threshold_idx to the end
                        reaction.energy = energy_grid[threshold_idx..].to_vec();
                        println!("Populated energy grid: {} points, threshold_idx: {}", 
                                 reaction.energy.len(), threshold_idx);
                    }
                }
            }
        }
    }
}