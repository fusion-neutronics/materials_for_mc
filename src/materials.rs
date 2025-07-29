use crate::nuclide::Nuclide;
use crate::material::Material;
use std::collections::HashMap;
use crate::nuclide::{Nuclide, get_or_load_nuclide};
use crate::config::CONFIG;
use std::sync::Arc;

/// A collection of materials that behaves like a list/vector
#[derive(Debug, Clone)]
pub struct Materials {
    /// Storage for materials in a vector
    materials: Vec<Material>,
    /// Loaded nuclide data (name -> Arc<Mutex<Nuclide>>)
    pub nuclide_data: HashMap<String, std::sync::Arc<std::sync::Mutex<Nuclide>>>,
}

impl Materials {
    /// Create a new empty materials collection
    pub fn new() -> Self {
        Materials {
            materials: Vec::new(),
            nuclide_data: HashMap::new(),
        }
    }

    /// Append a material to the collection (like a list)
    /// 
    /// # Arguments
    /// * `material` - The material to append
    pub fn append(&mut self, material: Material) {
        self.materials.push(material);
    }

    /// Get a reference to a material by index
    /// 
    /// # Returns
    /// * `Some(&Material)` if index is valid
    /// * `None` if index is out of bounds
    pub fn get(&self, index: usize) -> Option<&Material> {
        self.materials.get(index)
    }

    /// Get a mutable reference to a material by index
    /// 
    /// # Returns
    /// * `Some(&mut Material)` if index is valid
    /// * `None` if index is out of bounds
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Material> {
        self.materials.get_mut(index)
    }
    
    /// Remove a material at the specified index
    /// 
    /// # Returns
    /// * The removed material, or panics if index is out of bounds
    pub fn remove(&mut self, index: usize) -> Material {
        self.materials.remove(index)
    }
    
    /// Get the number of materials in the collection
    pub fn len(&self) -> usize {
        self.materials.len()
    }
    
    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }
    
    /// Get an iterator over the materials
    pub fn iter(&self) -> impl Iterator<Item = &Material> {
        self.materials.iter()
    }
    
    /// Get a mutable iterator over the materials
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Material> {
        self.materials.iter_mut()
    }

    /// Read nuclide data from JSON files for all materials, only once per nuclide
    pub fn read_nuclides_from_json(&mut self, nuclide_json_map: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        // Collect all unique nuclide names from all materials
        let mut needed: Vec<String> = Vec::new();
        for mat in &self.materials {
            for nuclide in mat.nuclides.keys() {
                if !needed.contains(nuclide) {
                    needed.push(nuclide.clone());
                }
            }
        }
        // Load each nuclide directly using get_or_load_nuclide
        for nuclide_name in &needed {
            let nuclide = get_or_load_nuclide(nuclide_name, nuclide_json_map)?;
            self.nuclide_data.insert(nuclide_name.clone(), Arc::clone(&nuclide));
        }
        // Ensure all materials reference the shared Arc<Nuclide> from self.nuclide_data
        for mat in &mut self.materials {
            mat.nuclide_data.clear();
            for nuclide_name in mat.nuclides.keys() {
                if let Some(shared_arc) = self.nuclide_data.get::<str>(nuclide_name) {
                    mat.nuclide_data.insert(nuclide_name.clone(), Arc::clone(shared_arc));
                }
            }
        }
        Ok(())
    }

    /// Ensure all nuclides for all materials are loaded, using the global configuration if needed
    pub fn ensure_nuclides_loaded(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Collect all unique nuclide names from all materials that aren't already loaded
        let mut needed: Vec<String> = Vec::new();
        for mat in &self.materials {
            for nuclide_name in mat.nuclides.keys() {
                if !self.nuclide_data.contains_key(nuclide_name) && !needed.contains(nuclide_name) {
                    needed.push(nuclide_name.clone());
                }
            }
        }
        
        if needed.is_empty() {
            return Ok(());
        }
        
        // Get the global configuration
        let config = CONFIG.lock().unwrap();
        
        // Load each missing nuclide
        for nuclide_name in &needed {
            let nuclide = get_or_load_nuclide(nuclide_name, &config.cross_sections)?;
            self.nuclide_data.insert(nuclide_name.clone(), Arc::clone(&nuclide));
        }
        
        // Ensure all materials reference the shared Arc<Nuclide> from self.nuclide_data
        for mat in &mut self.materials {
            for nuclide_name in mat.nuclides.keys() {
                if !mat.nuclide_data.contains_key(nuclide_name) {
                    if let Some(shared_arc) = self.nuclide_data.get::<str>(nuclide_name) {
                        mat.nuclide_data.insert(nuclide_name.clone(), Arc::clone(shared_arc));
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for Materials {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_materials() {
        let materials = Materials::new();
        assert!(materials.is_empty());
        assert_eq!(materials.len(), 0);
    }

    #[test]
    fn test_append_material() {
        let mut materials = Materials::new();
        let material = Material::new();
        
        materials.append(material);
        assert_eq!(materials.len(), 1);
    }

    #[test]
    fn test_get_material() {
        let mut materials = Materials::new();
        let mut material = Material::new();
        material.set_density("g/cm3", 10.5).unwrap();
        
        materials.append(material);
        let retrieved = materials.get(0);
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().density, Some(10.5));
    }

    #[test]
    fn test_remove_material() {
        let mut materials = Materials::new();
        let material = Material::new();
        
        materials.append(material);
        let _removed = materials.remove(0);
        
        assert!(materials.is_empty());
    }

    #[test]
    fn test_get_mut_material() {
        let mut materials = Materials::new();
        let material = Material::new();
        
        materials.append(material);
        
        // Modify the material through the mutable reference
        let material = materials.get_mut(0).unwrap();
        material.set_density("g/cm3", 10.5).unwrap();
        
        // Verify the modification was successful
        assert_eq!(materials.get(0).unwrap().density, Some(10.5));
    }

    #[test]
    fn test_materials_nuclide_arc_sharing() {
        use std::collections::HashMap;
        use std::sync::Arc;
        use crate::nuclide::Nuclide;

        // Prepare nuclide JSON map (adjust path as needed)
        let mut nuclide_json_map = HashMap::new();
        nuclide_json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());

        // Create a Materials collection and add many materials referencing the same nuclide
        let mut materials = super::Materials::new();
        for _ in 0..100 {
            let mut mat = crate::material::Material::new();
            mat.add_nuclide("Li6", 1.0).unwrap();
            materials.append(mat);
        }
        // Load nuclide data (should only load once and share Arc<Nuclide>)
        materials.read_nuclides_from_json(&nuclide_json_map).unwrap();

        // Collect all Arc pointers for Li6 from each material
        let mut arcs: Vec<Arc<Nuclide>> = Vec::new();
        for mat in &materials.materials {
            if let Some(arc) = mat.nuclide_data.get("Li6") {
                arcs.push(Arc::clone(arc));
            }
        }
        // All Arc pointers should point to the same allocation
        for i in 1..arcs.len() {
            assert!(Arc::ptr_eq(&arcs[0], &arcs[i]), "Nuclide Arc is not shared!");
        }
    }
}