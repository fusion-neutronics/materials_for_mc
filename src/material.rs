use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Material {
    /// Composition of the material as a map of nuclide names to their atomic fractions
    pub nuclides: HashMap<String, f64>,
    /// Density of the material in g/cm³
    pub density: Option<f64>,
    /// Density unit (default: g/cm³)
    pub density_unit: String,
}

impl Material {
    pub fn new() -> Self {
        Material {
            nuclides: HashMap::new(),
            density: None,
            density_unit: String::from("g/cm3"),
        }
    }

    pub fn add_nuclide(&mut self, nuclide: &str, fraction: f64) -> Result<(), String> {
        if fraction < 0.0 {
            return Err(String::from("Fraction cannot be negative"));
        }

        self.nuclides.insert(String::from(nuclide), fraction);
        Ok(())
    }

    pub fn set_density(&mut self, unit: &str, value: f64) -> Result<(), String> {
        if value <= 0.0 {
            return Err(String::from("Density must be positive"));
        }

        self.density = Some(value);
        self.density_unit = String::from(unit);
        Ok(())
    }

    // pub fn get_nuclide_fraction(&self, nuclide: &str) -> Option<f64> {
    //     self.nuclides.get(nuclide).cloned()
    // }

    // pub fn get_total_fraction(&self) -> f64 {
    //     self.nuclides.values().sum()
    // }

    // pub fn normalize(&mut self) -> Result<(), String> {
    //     let total = self.get_total_fraction();
    //     if total <= 0.0 {
    //         return Err(String::from("Cannot normalize: total fraction is zero or negative"));
    //     }

    //     for fraction in self.nuclides.values_mut() {
    //         *fraction /= total;
    //     }

    //     Ok(())
    // }
}