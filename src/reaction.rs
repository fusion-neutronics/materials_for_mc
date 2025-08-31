use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reaction {
    pub cross_section: Vec<f64>,
    pub threshold_idx: usize,
    pub interpolation: Vec<i32>,
    #[serde(skip, default)]
    pub energy: Vec<f64>,  // Reaction-specific energy grid
    pub mt_number: i32,  // The MT number for this reaction
}



impl Reaction {
    /// Returns the cross section value for a given neutron energy.
    /// If the energy is below the grid, returns the first value.
    /// If above, returns the last value.
    /// Otherwise, returns the value at the closest lower index.
    pub fn cross_section_at(&self, energy: f64) -> Option<f64> {
        if self.energy.is_empty() || self.cross_section.is_empty() {
            return None;
        }
        match self.energy.binary_search_by(|e| e.partial_cmp(&energy).unwrap()) {
            Ok(idx) => self.cross_section.get(idx).copied(),
            Err(idx) => {
                if idx == 0 {
                    self.cross_section.get(0).copied()
                } else if idx >= self.cross_section.len() {
                    self.cross_section.last().copied()
                } else {
                    self.cross_section.get(idx - 1).copied()
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_section_at() {
        let reaction = Reaction {
            cross_section: vec![1.0, 2.0, 3.0, 4.0],
            threshold_idx: 0,
            interpolation: vec![],
            energy: vec![0.5, 1.0, 2.0, 5.0],
            mt_number: 102,
        };

        // Below grid
        assert_eq!(reaction.cross_section_at(0.1), Some(1.0));
        // Exact match
        assert_eq!(reaction.cross_section_at(1.0), Some(2.0));
        // Between grid points
        assert_eq!(reaction.cross_section_at(1.5), Some(2.0));
        // Above grid
        assert_eq!(reaction.cross_section_at(10.0), Some(4.0));
    }
}
