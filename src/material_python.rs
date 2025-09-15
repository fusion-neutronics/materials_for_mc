use pyo3::prelude::*;
use crate::material::Material;
use crate::python_utils::convert_python_input_to_universal;
use pyo3::exceptions::PyValueError;
use std::collections::HashMap;

#[pyclass(name = "Material")]
pub struct PyMaterial {
    internal: Material,
}

#[pymethods]
impl PyMaterial {
    /// Sample a distance to the next neutron collision.
    ///
    /// Uses the macroscopic total cross section (calculating it first if missing)
    /// and an exponential distribution to sample a path length. A deterministic
    /// RNG seed can be supplied for reproducibility.
    ///
    /// Args:
    ///     energy (float): Neutron energy in eV.
    ///     seed (Optional[int]): RNG seed; if omitted a fixed internal seed is used.
    ///
    /// Returns:
    ///     Optional[float]: Sampled distance in cm, or None if total XS unavailable.
    #[pyo3(text_signature = "(self, energy, seed=None)")]
    fn sample_distance_to_collision(&self, energy: f64, seed: Option<u64>) -> Option<f64> {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::seed_from_u64(12345),
        };
        self.internal.sample_distance_to_collision(energy, &mut rng)
    }
    /// Create an empty material.
    ///
    /// Returns:
    ///     Material: A new material with no density or nuclides set.
    #[new]
    #[pyo3(text_signature = "()")]
    fn new() -> Self {
        PyMaterial {
            internal: Material::new(),
        }
    }

    /// Add (or update) a nuclide number fraction.
    ///
    /// Args:
    ///     nuclide (str): Nuclide name (e.g. "Fe56").
    ///     fraction (float): Number fraction (will be normalized with others later).
    ///
    /// Raises:
    ///     ValueError: On invalid fraction or name.
    #[pyo3(text_signature = "(self, nuclide, fraction)")]
    fn add_nuclide(&mut self, nuclide: String, fraction: f64) -> PyResult<()> {
        self.internal
            .add_nuclide(&nuclide, fraction)
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Set material density.
    ///
    /// Args:
    ///     unit (str): Density unit (e.g. "g/cm3", "atoms/b-cm").
    ///     value (float): Density value.
    ///
    /// Raises:
    ///     ValueError: If unit not supported.
    #[pyo3(text_signature = "(self, unit, value)")]
    fn set_density(&mut self, unit: String, value: f64) -> PyResult<()> {
        self.internal
            .set_density(&unit, value)
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Get the material nuclides as a tuple of (name, fraction) pairs
    #[getter]
    fn nuclides(&self) -> Vec<(String, f64)> {
        // Convert HashMap to a Vec of tuples
        let mut nuclide_vec: Vec<(String, f64)> = self
            .internal
            .nuclides
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        // Sort by nuclide name for consistent order
        nuclide_vec.sort_by(|a, b| a.0.cmp(&b.0));

        nuclide_vec
    }

    /// Material volume in cm^3, if set.
    #[getter]
    fn volume(&self) -> Option<f64> {
        self.internal.volume
    }

    /// Set the material volume (cm^3).
    #[setter]
    fn set_volume(&mut self, value: f64) -> PyResult<()> {
        self.internal
            .volume(Some(value))
            .map_err(|e| PyValueError::new_err(e))?;
        Ok(())
    }
    // Then try this:
    /// Return a list of nuclide names currently present in the material.
    #[pyo3(text_signature = "(self)")]
    fn get_nuclide_names(&self) -> Vec<String> {
        self.internal.get_nuclides()
    }

    /// String representation of the Material
    fn __str__(&self) -> PyResult<String> {
        let mut result = String::from("Material:\n");

        // Add density information
        if let Some(density) = self.internal.density {
            result.push_str(&format!(
                "  Density: {} {}\n",
                density, self.internal.density_units
            ));
        } else {
            result.push_str("  Density: not set\n");
        }

        // Add volume information
        if let Some(volume) = self.internal.volume {
            result.push_str(&format!("  Volume: {} cm³\n", volume));
        } else {
            result.push_str("  Volume: not set\n");
        }

        // Add nuclide information
        result.push_str("  Composition:\n");
        for (nuclide, fraction) in &self.internal.nuclides {
            result.push_str(&format!("    {}: {}\n", nuclide, fraction));
        }

        Ok(result)
    }

    /// Return the same string as __str__
    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    /// Density value (in current units) or None.
    #[getter]
    fn density(&self) -> Option<f64> {
        self.internal.density
    }

    /// Density units string.
    #[getter]
    fn density_units(&self) -> String {
        self.internal.density_units.clone()
    }

    #[pyo3(name = "read_nuclides_from_json")]
    /// Bulk load nuclide data from a mapping of nuclide -> JSON path or a keyword string.
    ///
    /// Args:
    ///     nuclide_json_map (Optional[Dict[str,str] | str]): Mapping of nuclide names to JSON file paths or a keyword string.
    ///
    /// Raises:
    ///     ValueError: If any JSON file cannot be read / parsed.
    fn read_nuclides_from_json(
        &mut self,
        _py: Python,
        nuclide_json_map: Option<&pyo3::types::PyAny>,
    ) -> PyResult<()> {
        let input = convert_python_input_to_universal(nuclide_json_map)?;
        self.internal.read_nuclides_universal(input)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Return raw pointer address of an internal shared Nuclide (debug only).
    ///
    /// Args:
    ///     nuclide (str): Nuclide name.
    ///
    /// Returns:
    ///     Optional[int]: Address value (process-local) or None if not present.
    fn nuclide_ptr_addr(&self, nuclide: &str) -> Option<usize> {
        self.internal.nuclide_data.get(nuclide).map(|arc| {
            let ptr: *const crate::nuclide::Nuclide = std::sync::Arc::as_ptr(arc);
            ptr as usize
        })
    }

    /// Temperature label (e.g. "293K").
    #[getter]
    fn temperature(&self) -> String {
        self.internal.temperature.clone()
    }

    /// Set current temperature label.
    #[setter]
    fn set_temperature(&mut self, temperature: &str) {
        self.internal.set_temperature(temperature);
    }

    /// Return (and build if needed) the unified neutron energy grid.
    ///
    /// Returns:
    ///     List[float]: Energy grid in eV.
    #[pyo3(text_signature = "(self)")]
    fn unified_energy_grid_neutron(&mut self) -> Vec<f64> {
        self.internal.unified_energy_grid_neutron()
    }

    /// Calculate microscopic neutron cross sections on the unified energy grid.
    ///
    /// Args:
    ///     mt_filter (Optional[List[int]]): Restrict to these MT numbers.
    ///
    /// Returns:
    ///     Dict[str, Dict[int, List[float]]]: nuclide -> MT -> xs array
    #[pyo3(signature = (mt_filter=None))]
    fn calculate_microscopic_xs_neutron(
        &mut self,
        mt_filter: Option<Vec<i32>>,
    ) -> HashMap<String, HashMap<i32, Vec<f64>>> {
        self.internal
            .calculate_microscopic_xs_neutron(mt_filter.as_ref())
    }

    /// Calculate macroscopic neutron cross sections (total or subset of MTs).
    ///
    /// Builds the unified energy grid if not already present.
    ///
    /// Args:
    ///     mt_filter (Optional[List[int]]): MT numbers to include (default [1] total).
    ///     by_nuclide (bool): If True, store per-nuclide macroscopic XS internally.
    ///
    /// Returns:
    ///     Tuple[List[float], Dict[int, List[float]]]: (energy grid, MT -> Σ array)
    #[pyo3(signature = (mt_filter = None, by_nuclide = false))]
    fn calculate_macroscopic_xs_neutron(
        &mut self,
        mt_filter: Option<Vec<i32>>,
        by_nuclide: bool,
    ) -> (Vec<f64>, HashMap<i32, Vec<f64>>) {
        let default_mt = vec![1];
        let mt_vec: &Vec<i32> = match mt_filter.as_ref() {
            Some(v) => v,
            None => &default_mt,
        };
        let (energy_grid, xs_dict_i32) = self
            .internal
            .calculate_macroscopic_xs_neutron(mt_vec, by_nuclide);
        (energy_grid, xs_dict_i32)
    }

    /// Cached macroscopic neutron cross sections (MT -> Σ(E)).
    #[getter]
    fn macroscopic_xs_neutron(&self) -> HashMap<i32, Vec<f64>> {
        self.internal.macroscopic_xs_neutron.clone()
    }

    /// Number density (atoms / barn-cm) per nuclide.
    ///
    /// Returns:
    ///     Dict[str, float]: nuclide -> atoms / b-cm
    fn get_atoms_per_barn_cm(&self) -> HashMap<String, f64> {
        self.internal.get_atoms_per_barn_cm()
    }

    /// Compute neutron mean free path at a given energy.
    ///
    /// Args:
    ///     energy (float): Neutron energy in eV.
    ///
    /// Returns:
    ///     Optional[float]: Mean free path (cm) or None if total XS unavailable.
    fn mean_free_path_neutron(&mut self, energy: f64) -> Option<f64> {
        self.internal.mean_free_path_neutron(energy)
    }

    /// Add a natural element by atomic fraction (expands to isotopes internally).
    ///
    /// Args:
    ///     element (str): Element symbol (e.g. "Fe").
    ///     fraction (float): Atomic fraction for the element.
    #[pyo3(text_signature = "(self, element, fraction)")]
    fn add_element(&mut self, element: String, fraction: f64) -> PyResult<()> {
        self.internal
            .add_element(&element, fraction)
            .map_err(|e| PyValueError::new_err(e))
    }

    /// Sorted list of all unique MT reaction numbers present.
    #[getter]
    fn reaction_mts(&mut self) -> PyResult<Vec<i32>> {
        self.internal
            .reaction_mts()
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
    /// Sample which nuclide undergoes an interaction at a given energy.
    ///
    /// Uses per-nuclide macroscopic total cross sections as weights.
    ///
    /// Args:
    ///     energy (float): Neutron energy in eV.
    ///     seed (Optional[int]): RNG seed for reproducibility.
    ///
    /// Returns:
    ///     str: Selected nuclide name.
    #[pyo3(signature = (energy, seed=None))]
    fn sample_interacting_nuclide(&self, energy: f64, seed: Option<u64>) -> PyResult<String> {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::seed_from_u64(12345),
        };
        Ok(self.internal.sample_interacting_nuclide(energy, &mut rng))
    }
}

// Add these helper methods in a separate impl block
impl PyMaterial {
    // Helper method for other Python modules to access the internal Material
    pub(crate) fn get_internal(&self) -> &Material {
        &self.internal
    }

    // Helper method to create a PyMaterial from a Material
    pub(crate) fn from_material(material: Material) -> Self {
        PyMaterial { internal: material }
    }
}
