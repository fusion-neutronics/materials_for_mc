use crate::material::Material;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use crate::nuclide::{Nuclide, read_nuclide_from_json};

extern crate once_cell;

static GLOBAL_NUCLIDE_CACHE: Lazy<Mutex<HashMap<String, Nuclide>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// A collection of materials that behaves like a list/vector
#[derive(Debug, Clone)]
pub struct Materials {
    /// Storage for materials in a vector
    materials: Vec<Material>,
    /// Loaded nuclide data (name -> Nuclide)
    pub nuclide_data: HashMap<String, Nuclide>,
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
        let mut needed: Vec<String> = Vec::new();
        for mat in &self.materials {
            for nuclide in mat.nuclides.keys() {
                if !needed.contains(nuclide) {
                    needed.push(nuclide.clone());
                }
            }
        }
        let mut cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        for nuclide in needed {
            if !cache.contains_key(&nuclide) {
                if let Some(path) = nuclide_json_map.get(&nuclide) {
                    let data = read_nuclide_from_json(path)?;
                    cache.insert(nuclide.clone(), data);
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
        let removed = materials.remove(0);
        
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
}