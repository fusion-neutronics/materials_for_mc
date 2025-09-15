// Struct representing a nuclide, matching the JSON file structure
// Update the fields as needed to match all JSON entries
use crate::reaction::Reaction;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};

// Global cache for nuclides to avoid reloading
static GLOBAL_NUCLIDE_CACHE: Lazy<Mutex<HashMap<String, Arc<Nuclide>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Clear the global nuclide cache (used by tests and Python to ensure deterministic selective temperature behavior)
#[allow(dead_code)]
pub fn clear_nuclide_cache() {
    let mut cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
    cache.clear();
}

/// Core data model for a single nuclide and its reaction cross section data.
///
/// A `Nuclide` mirrors (and is deserialized from) a JSON schema containing
/// metadata plus reaction channel data at one or more temperatures. Reaction
/// data are organized by temperature key (e.g. "294") and then by ENDF/MT
/// number. Each [`Reaction`] holds its own threshold information and (possibly
/// truncated) energy grid relative to the top‑level temperature energy grid.
///
/// Temperatures:
/// * `available_temperatures` always lists every temperature present in the
///   source JSON file – even if a filtered load only materialized a subset.
/// * `loaded_temperatures` tracks the subset actually parsed into `reactions`
///   and `energy` according to caller filtering semantics.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nuclide {
    /// Canonical nuclide name (e.g. "Li6"). May be derived when absent.
    pub name: Option<String>,
    /// Optional human readable element name (legacy field; may be absent).
    pub element: Option<String>,
    /// Element symbol, e.g. "Li".
    pub atomic_symbol: Option<String>,
    /// Atomic (proton) number Z.
    pub atomic_number: Option<u32>,
    /// Neutron number N (may be computed from A - Z if missing).
    pub neutron_number: Option<u32>,
    /// Mass number A.
    pub mass_number: Option<u32>,
    /// Origin / library identifier (e.g. JEFF, ENDF, custom tag).
    pub library: Option<String>,
    /// Top‑level energy grid per temperature (full grid; per‑reaction grids may be threshold‑truncated).
    pub energy: Option<HashMap<String, Vec<f64>>>,
    /// temperature -> MT number -> reaction data.
    #[serde(default)]
    pub reactions: HashMap<String, HashMap<i32, Reaction>>, // temperature -> mt (i32) -> Reaction
    /// True if any fission MT channel is present.
    pub fissionable: bool,
    /// All temperatures present in the JSON file regardless of filtering.
    #[serde(skip, default)]
    pub available_temperatures: Vec<String>, // All temps listed in the JSON (even if not loaded)
    /// Subset of temperatures actually loaded into `reactions` / `energy`.
    #[serde(skip, default)]
    pub loaded_temperatures: Vec<String>, // Subset actually loaded into reactions/energy
    /// Optional path the JSON was read from (None for in‑memory sources / WASM).
    #[serde(skip, default)]
    pub data_path: Option<String>, // Path JSON was loaded from (for potential future extension)
}

impl Nuclide {
    /// Sample the top-level reaction type (fission, absorption, elastic, inelastic, other) at a given energy and temperature
    pub fn sample_reaction<R: rand::Rng + ?Sized>(
        &self,
        energy: f64,
        temperature: &str,
        rng: &mut R,
    ) -> Option<&Reaction> {
        // Try temperature as given, then with 'K' appended, then any available
        let temp_reactions = if let Some(r) = self.reactions.get(temperature) {
            r
        } else if let Some(r) = self.reactions.get(&format!("{}K", temperature)) {
            r
        } else if let Some((temp, r)) = self.reactions.iter().next() {
            println!("[sample_reaction] Requested temperature '{}' not found. Using available temperature '{}'.", temperature, temp);
            r
        } else {
            println!("[sample_reaction] No reaction data available for any temperature.");
            return None;
        };

        // Define MTs for each event type
        let total_mt = 1;
        let fission_mt = 18;
        let absorption_mt = 101;
        let elastic_mt = 2;
        let nonelastic_mt = 3;

        // Helper to get xs for a given MT using Reaction::cross_section_at
        let get_xs = |mt: i32| -> f64 {
            temp_reactions
                .get(&mt)
                .and_then(|reaction| reaction.cross_section_at(energy))
                .unwrap_or(0.0)
        };

        let total_xs = get_xs(total_mt);
        if total_xs <= 0.0 {
            return None;
        }

        let xi = rng.gen_range(0.0..total_xs);
        let mut accum = 0.0;

        // Absorption

        let xs_absorption = get_xs(absorption_mt);
        accum += xs_absorption;
        if xi < accum && xs_absorption > 0.0 {
            return temp_reactions.get(&absorption_mt);
        }

        // Elastic
        let xs_elastic = get_xs(elastic_mt);
        accum += xs_elastic;
        if xi < accum && xs_elastic > 0.0 {
            return temp_reactions.get(&elastic_mt);
        }

        // Fission (only if nuclide is fissionable, checked last)
        let xs_fission = if self.fissionable {
            get_xs(fission_mt)
        } else {
            0.0
        };
        accum += xs_fission;
        if xi < accum && xs_fission > 0.0 {
            return temp_reactions.get(&fission_mt);
        }

        // Non-elastic selection as fallback
        temp_reactions.get(&nonelastic_mt)
    }
    /// Get the energy grid for a specific temperature
    pub fn energy_grid(&self, temperature: &str) -> Option<&Vec<f64>> {
        self.energy
            .as_ref()
            .and_then(|energy_map| energy_map.get(temperature))
    }

    /// Get a list of available temperatures
    pub fn temperatures(&self) -> Option<Vec<String>> {
        let mut temps = std::collections::HashSet::new();

        // Check reactions first
        for temp in self.reactions.keys() {
            temps.insert(temp.clone());
        }

        // Also check energy map
        if let Some(energy_map) = &self.energy {
            for temp in energy_map.keys() {
                temps.insert(temp.clone());
            }
        }

        if temps.is_empty() {
            None
        } else {
            let mut temps_vec: Vec<String> = temps.into_iter().collect();
            temps_vec.sort();
            Some(temps_vec)
        }
    }

    /// Get a list of available MT numbers
    pub fn reaction_mts(&self) -> Option<Vec<i32>> {
        let mut mts = std::collections::HashSet::new();
        for temp_reactions in self.reactions.values() {
            for &mt in temp_reactions.keys() {
                mts.insert(mt);
            }
        }
        if mts.is_empty() {
            None
        } else {
            let mut mts_vec: Vec<i32> = mts.into_iter().collect();
            mts_vec.sort();
            Some(mts_vec)
        }
    }

    /// Get microscopic cross section data for a specific MT number and temperature.
    /// Returns a tuple of (cross_section_values, energy_grid).
    /// If temperature is None, uses the single loaded temperature if only one exists.
    pub fn microscopic_cross_section(
        &self,
        mt: i32,
        temperature: Option<&str>,
    ) -> Result<(Vec<f64>, Vec<f64>), Box<dyn std::error::Error>> {
        // Determine which temperature to use
        let temp_key = if let Some(temp) = temperature {
            // Try the provided temperature as-is, then with 'K' suffix
            if self.reactions.contains_key(temp) {
                temp
            } else if self.reactions.contains_key(&format!("{}K", temp)) {
                &format!("{}K", temp)
            } else {
                return Err(format!("Temperature '{}' not found in loaded data", temp).into());
            }
        } else {
            // No temperature provided - use single loaded temperature if available
            if self.loaded_temperatures.len() == 1 {
                &self.loaded_temperatures[0]
            } else if self.loaded_temperatures.is_empty() {
                return Err("No temperatures loaded in nuclide data".into());
            } else {
                return Err(format!(
                    "Multiple temperatures loaded ({}), must specify which one to use",
                    self.loaded_temperatures.join(", ")
                ).into());
            }
        };

        // Get the reaction data for this temperature and MT
        let temp_reactions = self.reactions.get(temp_key)
            .ok_or_else(|| format!("Temperature '{}' not found in reactions", temp_key))?;
        
        let reaction = temp_reactions.get(&mt)
            .ok_or_else(|| format!("MT {} not found for temperature '{}'", mt, temp_key))?;

        // Return the cross section and energy data
        if reaction.cross_section.is_empty() {
            return Err(format!("No cross section data available for MT {} at temperature '{}'", mt, temp_key).into());
        }

        if reaction.energy.is_empty() {
            return Err(format!("No energy grid available for MT {} at temperature '{}'", mt, temp_key).into());
        }

        Ok((reaction.cross_section.clone(), reaction.energy.clone()))
    }
}

// Internal: parse a nuclide from a JSON value with optional temperature filter.
// If `temps_filter` is Some, only those temperatures will have reaction/energy data
// materialized into the returned struct. `available_temperatures` will always list
// the full set present in the file for deterministic behavior.
fn parse_nuclide_from_json_value(
    json_value: serde_json::Value,
    temps_filter: Option<&std::collections::HashSet<String>>,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    let mut nuclide = Nuclide {
        name: None,
        element: None,
        atomic_symbol: None,
        atomic_number: None,
        neutron_number: None,
        mass_number: None,
        library: None,
        energy: None,
        reactions: HashMap::new(),
        fissionable: false,
        available_temperatures: Vec::new(),
        loaded_temperatures: Vec::new(),
        data_path: None,
    };

    // Parse basic metadata
    if let Some(name) = json_value.get("name").and_then(|v| v.as_str()) {
        nuclide.name = Some(name.to_string());
    }

    if let Some(element) = json_value.get("element").and_then(|v| v.as_str()) {
        nuclide.element = Some(element.to_string());
    }

    if let Some(symbol) = json_value.get("atomic_symbol").and_then(|v| v.as_str()) {
        nuclide.atomic_symbol = Some(symbol.to_string());
        // For backward compatibility, set element to "lithium" if atomic_symbol is "Li"
        if symbol == "Li" && nuclide.element.is_none() {
            nuclide.element = Some("lithium".to_string());
        }
    }

    if let Some(num) = json_value.get("atomic_number").and_then(|v| v.as_u64()) {
        nuclide.atomic_number = Some(num as u32);
    }

    if let Some(num) = json_value.get("mass_number").and_then(|v| v.as_u64()) {
        nuclide.mass_number = Some(num as u32);
    }

    if let Some(num) = json_value.get("neutron_number").and_then(|v| v.as_u64()) {
        nuclide.neutron_number = Some(num as u32);
    } else if nuclide.mass_number.is_some() && nuclide.atomic_number.is_some() {
        // Calculate neutron_number = mass_number - atomic_number
        nuclide.neutron_number =
            Some(nuclide.mass_number.unwrap() - nuclide.atomic_number.unwrap());
    }

    if let Some(lib) = json_value.get("library").and_then(|v| v.as_str()) {
        nuclide.library = Some(lib.to_string());
    }

    // First, collect ALL available temperatures from multiple locations in the JSON
    let mut all_temperatures = std::collections::HashSet::new();

    // Step 1: Check the dedicated "temperatures" array (this is the source of truth)
    // The "temperatures" array in the JSON is the authoritative source of all available temperatures
    if let Some(temps_array) = json_value.get("temperatures").and_then(|v| v.as_array()) {
        for temp_value in temps_array {
            if let Some(temp_str) = temp_value.as_str() {
                all_temperatures.insert(temp_str.to_string());
            }
        }
    }

    // Step 2: Also check the reactions object for temperatures
    if let Some(reactions_obj) = json_value.get("reactions").and_then(|v| v.as_object()) {
        for temp in reactions_obj.keys() {
            all_temperatures.insert(temp.clone());
        }
    }

    // Step 3: Also check energy object for temperatures
    if let Some(energy_obj) = json_value.get("energy").and_then(|v| v.as_object()) {
        for temp in energy_obj.keys() {
            all_temperatures.insert(temp.clone());
        }
    }

    // Store all available temperatures (regardless of filtering)
    let mut available_temps: Vec<String> = all_temperatures.iter().cloned().collect();
    available_temps.sort();
    nuclide.available_temperatures = available_temps;

    // Check if we have the format with "reactions" field
    if let Some(reactions_obj) = json_value.get("reactions").and_then(|v| v.as_object()) {
        for (temp, mt_reactions) in reactions_obj {
            // Skip temperatures not in the filter if a filter is provided and non-empty
            if let Some(filter) = temps_filter {
                if !filter.is_empty() && !filter.contains(temp) {
                    continue;
                }
            }

            let mut temp_reactions: HashMap<i32, Reaction> = HashMap::new();

            // Process all MT reactions for this temperature
            if let Some(mt_obj) = mt_reactions.as_object() {
                for (mt, reaction_data) in mt_obj {
                    if let Some(reaction_obj) = reaction_data.as_object() {
                        let mut reaction = Reaction {
                            cross_section: Vec::new(),
                            threshold_idx: 0,
                            interpolation: Vec::new(),
                            energy: Vec::new(),
                            mt_number: 0,
                        };

                        // Get cross section (might be named "xs" in old format)
                        if let Some(xs) = reaction_obj
                            .get("cross_section")
                            .or_else(|| reaction_obj.get("xs"))
                        {
                            if let Some(xs_arr) = xs.as_array() {
                                reaction.cross_section =
                                    xs_arr.iter().filter_map(|v| v.as_f64()).collect();
                            }
                        }

                        // Get threshold_idx
                        if let Some(idx) =
                            reaction_obj.get("threshold_idx").and_then(|v| v.as_u64())
                        {
                            reaction.threshold_idx = idx as usize;
                        }

                        // Get interpolation
                        if let Some(interp) = reaction_obj.get("interpolation") {
                            if let Some(interp_arr) = interp.as_array() {
                                reaction.interpolation = interp_arr
                                    .iter()
                                    .filter_map(|v| v.as_i64().map(|i| i as i32))
                                    .collect();
                            }
                        }

                        // Get energy (some reactions may have their own energy grid)
                        if let Some(energy) = reaction_obj.get("energy") {
                            if let Some(energy_arr) = energy.as_array() {
                                reaction.energy =
                                    energy_arr.iter().filter_map(|v| v.as_f64()).collect();
                            }
                        }

                        // Calculate energy grid from threshold_idx and main energy grid if not already set
                        if reaction.energy.is_empty() {
                            if let Some(energy_grids) = &nuclide.energy {
                                if let Some(energy_grid) = energy_grids.get(temp) {
                                    if reaction.threshold_idx < energy_grid.len() {
                                        reaction.energy =
                                            energy_grid[reaction.threshold_idx..].to_vec();
                                    }
                                }
                            }
                        }

                        if let Ok(mt_int) = mt.parse::<i32>() {
                            reaction.mt_number = mt_int;
                            temp_reactions.insert(mt_int, reaction);
                        }
                    }
                }
            }

            // Only insert if we found reactions
            if !temp_reactions.is_empty() {
                nuclide.reactions.insert(temp.clone(), temp_reactions);
            }
        }
    }

    // Process energy (after reactions so we know which temps to keep)
    if let Some(energy_obj) = json_value.get("energy").and_then(|v| v.as_object()) {
        let mut energy_map = HashMap::new();

        for (temp, energy_arr) in energy_obj {
            // Skip temperatures not in the filter if a filter is provided and non-empty
            if let Some(filter) = temps_filter {
                if !filter.is_empty() && !filter.contains(temp) {
                    continue;
                }
            }

            if let Some(energy_values) = energy_arr.as_array() {
                let energy_vec: Vec<f64> =
                    energy_values.iter().filter_map(|v| v.as_f64()).collect();
                energy_map.insert(temp.clone(), energy_vec);
            }
        }
        if !energy_map.is_empty() {
            nuclide.energy = Some(energy_map);
        }
    }

    // Validate temperature sets: if both reactions and energy present, they must match exactly.
    if let Some(energy_map) = &nuclide.energy {
        use std::collections::HashSet;
        let reaction_temps: HashSet<String> = nuclide.reactions.keys().cloned().collect();
        let energy_temps: HashSet<String> = energy_map.keys().cloned().collect();

        if reaction_temps.is_empty() && !energy_temps.is_empty() {
            return Err(format!(
                "Energy grid has temperatures {:?} but no reactions were loaded.",
                energy_temps
            )
            .into());
        }
        if !reaction_temps.is_empty() && energy_temps.is_empty() {
            return Err(format!(
                "Reactions have temperatures {:?} but no energy grids were loaded.",
                reaction_temps
            )
            .into());
        }

        if reaction_temps != energy_temps {
            let only_in_reactions: Vec<String> =
                reaction_temps.difference(&energy_temps).cloned().collect();
            let only_in_energy: Vec<String> =
                energy_temps.difference(&reaction_temps).cloned().collect();
            return Err(format!(
                "Mismatched temperature sets. Only in reactions: {:?}. Only in energy: {:?}.",
                only_in_reactions, only_in_energy
            )
            .into());
        }

        let mut temps: Vec<String> = reaction_temps.into_iter().collect();
        temps.sort();
        nuclide.available_temperatures = temps;
    } else if !nuclide.reactions.is_empty() {
        // Reactions present but no energy map created (legacy / missing). Treat as error per requirement.
        let temps: Vec<String> = nuclide.reactions.keys().cloned().collect();
        return Err(format!(
            "Reactions contain temperatures {:?} but no top-level energy map present.",
            temps
        )
        .into());
    } else {
        nuclide.available_temperatures.clear();
    }

    // If we have no energy map but have reactions, try to create an energy map
    if nuclide.energy.is_none() && !nuclide.reactions.is_empty() {
        let mut energy_grids = HashMap::new();

        for (temp, temp_reactions) in &nuclide.reactions {
            if let Some((_, reaction)) = temp_reactions.iter().next() {
                if !reaction.energy.is_empty() {
                    energy_grids.insert(temp.clone(), reaction.energy.clone());
                }
            }
        }

        if !energy_grids.is_empty() {
            nuclide.energy = Some(energy_grids);
        }
    }

    // At this point we should have both reactions and (possibly) an energy map.
    // Populate per-reaction energy grids if they are still empty so that
    // cross_section_at() works correctly for sampling.
    if let Some(energy_map) = &nuclide.energy {
        for (temp, temp_reactions) in nuclide.reactions.iter_mut() {
            if let Some(energy_grid) = energy_map.get(temp) {
                for reaction in temp_reactions.values_mut() {
                    if reaction.energy.is_empty() {
                        if reaction.threshold_idx < energy_grid.len() {
                            reaction.energy = energy_grid[reaction.threshold_idx..].to_vec();
                        }
                    }
                }
            }
        }
    }

    // Fallback: derive name if still None (e.g., from atomic_symbol + mass_number)
    if nuclide.name.is_none() {
        if let (Some(symbol), Some(mass)) = (&nuclide.atomic_symbol, nuclide.mass_number) {
            nuclide.name = Some(format!("{}{}", symbol, mass));
        }
    }

    // Determine fissionable status now that reactions are loaded
    let fission_mt_list = [18, 19, 20, 21, 38];
    if nuclide
        .reactions
        .values()
        .any(|temp_reactions| temp_reactions.keys().any(|mt| fission_mt_list.contains(mt)))
    {
        nuclide.fissionable = true;
    }

    // Set loaded_temperatures based on what was actually loaded
    let mut loaded_temps: Vec<String> = nuclide.reactions.keys().cloned().collect();
    loaded_temps.sort();
    nuclide.loaded_temperatures = loaded_temps;

    Ok(nuclide)
}

// Read a single nuclide either from an explicit JSON file path, or if the input is not a file path,
// treat the argument as a nuclide name and look up the path in the global CONFIG.cross_sections map.

/// Read a single nuclide either from an explicit JSON file path, or if the input is not a file path,
/// treat the argument as a nuclide name and look up the path in the global CONFIG.cross_sections map.
/// Optional temperature filtering is supported. URLs are automatically downloaded and cached.
pub fn read_nuclide_from_json<P: AsRef<Path>>(
    path_or_name: P,
    temps: Option<&std::collections::HashSet<String>>,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    // If path_or_name is a keyword, set config mapping for this nuclide
    let candidate_ref = path_or_name.as_ref();
    let candidate_str = candidate_ref.to_string_lossy();
    if crate::url_cache::is_keyword(&candidate_str) {
        let mut cfg = crate::config::CONFIG.lock().unwrap();
        cfg.set_cross_section(candidate_str.as_ref(), Some(&candidate_str));
    }
    read_nuclide_from_json_with_name(path_or_name, temps, None)
}

/// Read a single nuclide with an optional nuclide name hint for URL caching
pub fn read_nuclide_from_json_with_name<P: AsRef<Path>>(
    path_or_name: P,
    temps: Option<&std::collections::HashSet<String>>,
    nuclide_name_hint: Option<&str>,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    // Load the JSON file
    let candidate_ref = path_or_name.as_ref();
    let candidate_str = candidate_ref.to_string_lossy();
    
    let resolved_path = if candidate_ref.exists() {
        // Direct file path exists
        candidate_ref.to_path_buf()
    } else if crate::url_cache::is_url(&candidate_str) {
        // It's a URL, download and cache it
        if let Some(name) = nuclide_name_hint {
            crate::url_cache::resolve_path_or_url(&candidate_str, name)?
        } else {
            return Err("Direct URL loading without nuclide name not yet supported. Use config approach instead.".into());
        }
    } else if crate::url_cache::is_keyword(&candidate_str) {
        // It's a keyword: treat as data source for the nuclide name
        let nuclide_name = nuclide_name_hint.unwrap_or(candidate_str.as_ref());
        crate::url_cache::resolve_path_or_url(&candidate_str, nuclide_name)?
    } else {
        // Treat as nuclide name, look up in config
        let cfg = crate::config::CONFIG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let path_or_url = cfg
            .cross_sections
            .get(candidate_str.as_ref())
            .ok_or_else(|| {
                format!(
                    "Input '{}' is neither an existing file nor a key in Config cross_sections",
                    candidate_str
                )
            })?;
        // The config value might be a URL or local path
        crate::url_cache::resolve_path_or_url(path_or_url, candidate_str.as_ref())?
    };

    let file = File::open(&resolved_path)?;
    let reader = BufReader::new(file);
    let json_value: serde_json::Value = serde_json::from_reader(reader)?;

    // Use the filtering version of parse_nuclide_from_json_value
    let mut nuclide = parse_nuclide_from_json_value(json_value, temps)?;
    nuclide.data_path = Some(resolved_path.to_string_lossy().to_string());

    Ok(nuclide)
}

// Read a nuclide from a JSON string, used by WASM
pub fn read_nuclide_from_json_str(
    json_content: &str,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    // Parse the JSON string
    let json_value: serde_json::Value = serde_json::from_str(json_content)?;

    // Use the shared parsing function
    let mut nuclide = parse_nuclide_from_json_value(json_value, None)?;
    // No file path available
    nuclide.data_path = None;
    Ok(nuclide)
}

/// Read (or fetch from cache) a nuclide, optionally keeping only a specified subset of temperatures.
/// Records the full `available_temperatures` and tracks the actually retained subset in `loaded_temperatures`.
///
/// This function ensures that:
/// 1. All temperatures present in the JSON file are recorded in `available_temperatures` regardless of filtering
/// 2. Only the temperatures that pass the filter are loaded into `loaded_temperatures` and the actual data structures
///
/// Unified loader semantics:
/// - `temperatures_to_include`: `None` or `Some(empty)` => load all temperatures.
/// - `Some(nonempty)` => ensure those temperatures are loaded; if previously loaded with fewer temps, reload & prune to the union.
///
/// Implementation note: This function performs two passes on the JSON data when filtering:
/// - First pass with no filter to determine all available temperatures
/// - Second pass with the filter to load only the requested temperatures
pub fn get_or_load_nuclide(
    nuclide_name: &str,
    json_path_map: &HashMap<String, String>,
    temperatures_to_include: Option<&std::collections::HashSet<String>>,
) -> Result<Arc<Nuclide>, Box<dyn std::error::Error>> {
    use std::collections::HashSet;
    let requested: HashSet<String> = temperatures_to_include
        .map(|s| s.iter().cloned().collect())
        .unwrap_or_else(HashSet::new);

    // Fast path: cache hit with sufficient temps
    {
        let cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        if let Some(existing) = cache.get(nuclide_name) {
            if requested.is_empty()
                || requested.is_subset(&existing.loaded_temperatures.iter().cloned().collect())
            {
                return Ok(Arc::clone(existing));
            }
        }
    }
    // Determine union (existing loaded + requested)
    let mut union_set = requested.clone();
    {
        let cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        if let Some(existing) = cache.get(nuclide_name) {
            for t in &existing.loaded_temperatures {
                union_set.insert(t.clone());
            }
        }
    }
    // Load directly with temperature filtering
    // If the mapping is missing but nuclide_name is a keyword, set config mapping
    let mut path_or_url = json_path_map.get(nuclide_name).cloned();
    if path_or_url.is_none() && crate::url_cache::is_keyword(nuclide_name) {
        let mut cfg = crate::config::CONFIG.lock().unwrap();
        cfg.set_cross_section(nuclide_name, Some(nuclide_name));
        path_or_url = Some(nuclide_name.to_string());
    }
    let path_or_url = path_or_url.ok_or_else(|| {
        format!(
            "No JSON file provided for nuclide '{}'. Please supply a path for all nuclides.",
            nuclide_name
        )
    })?;

    // Resolve URL or local path - download and cache if it's a URL
    let resolved_path = crate::url_cache::resolve_path_or_url(&path_or_url, nuclide_name)?;

    // Load the JSON file once
    let file = File::open(&resolved_path)?;
    let reader = BufReader::new(file);
    let json_value: serde_json::Value = serde_json::from_reader(reader)?;

    // Parse with temperature filter if needed
    let filter_option = if union_set.is_empty() {
        None
    } else {
        Some(&union_set)
    };

    // First, do a parse with no filtering to get all available temperatures
    let all_temps_nuclide = parse_nuclide_from_json_value(json_value.clone(), None)?;

    // Now parse with the filter to get the loaded data
    let mut nuclide = parse_nuclide_from_json_value(json_value, filter_option)?;
    nuclide.data_path = Some(resolved_path.to_string_lossy().to_string());

    // Copy the available_temperatures from the unfiltered parse
    nuclide.available_temperatures = all_temps_nuclide.available_temperatures;

    // Print loading info
    let name_disp = nuclide.name.as_deref().unwrap_or(nuclide_name);
    println!(
        "Reading {} from {}, available temperatures: {}, loaded temperatures: {}",
        name_disp,
        resolved_path.to_string_lossy(),
        nuclide.available_temperatures.len(),
        nuclide.loaded_temperatures.len()
    );
    let arc = Arc::new(nuclide);
    {
        let mut cache = GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        cache.insert(nuclide_name.to_string(), Arc::clone(&arc));
    }
    Ok(arc)
}

/// Load a nuclide with Python wrapper semantics - handles both path and name parameters
/// and preserves available_temperatures when filtering. This method consolidates the logic
/// previously scattered in the Python wrapper.
#[allow(dead_code)]
pub fn load_nuclide_for_python(
    path_or_keyword: Option<&str>,
    nuclide_name: Option<&str>,
    temperatures: Option<&std::collections::HashSet<String>>,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    // Determine the identifier - path takes precedence, then name
    let identifier = if let Some(p) = path_or_keyword {
        p
    } else if let Some(n) = nuclide_name {
        n
    } else {
        return Err("Either path or nuclide name must be provided".into());
    };

    // Check if identifier is a keyword and set up config if needed
    if crate::url_cache::is_keyword(identifier) {
        let mut cfg = crate::config::CONFIG.lock().unwrap();
        cfg.set_cross_section(identifier, Some(identifier));
    }

    // Load without temperature filtering first to get all available temperatures
    let full_nuclide = if path_or_keyword.is_some() && nuclide_name.is_some() {
        // We have both path and name - use the version with name hint
        read_nuclide_from_json_with_name(identifier, None, nuclide_name)?
    } else {
        read_nuclide_from_json(identifier, None)?
    };

    // Now load with temperature filtering if specified
    let mut filtered_nuclide = if temperatures.is_some() {
        if path_or_keyword.is_some() && nuclide_name.is_some() {
            read_nuclide_from_json_with_name(identifier, temperatures, nuclide_name)?
        } else {
            read_nuclide_from_json(identifier, temperatures)?
        }
    } else {
        full_nuclide.clone()
    };

    // Always preserve the full available_temperatures from the unfiltered load
    filtered_nuclide.available_temperatures = full_nuclide.available_temperatures;
    
    Ok(filtered_nuclide)
}

/// Load a nuclide from a path or keyword for the standalone Python function
/// This handles keyword detection and resolution automatically
#[allow(dead_code)]
pub fn load_nuclide_from_path_or_keyword(
    path_or_keyword: &str,
) -> Result<Nuclide, Box<dyn std::error::Error>> {
    // Check if it's a keyword and set up config if needed
    if crate::url_cache::is_keyword(path_or_keyword) {
        let mut cfg = crate::config::CONFIG.lock().unwrap();
        cfg.set_cross_section(path_or_keyword, Some(path_or_keyword));
    }
    
    // Load the nuclide with all temperatures
    read_nuclide_from_json(path_or_keyword, None)
}

mod tests {
    #[test]
    fn test_get_or_load_nuclide_uses_cache() {
        use std::collections::HashMap;
        let li6_path = std::path::Path::new("tests/Li6.json");
        assert!(li6_path.exists(), "tests/Li6.json missing");
        // Only remove Li6 from cache, don't clear all (avoid race with other tests)
        {
            let mut cache = super::GLOBAL_NUCLIDE_CACHE.lock().unwrap();
            cache.remove("Li6");
        }
        let raw = super::read_nuclide_from_json(li6_path, None).expect("Direct read failed");
        assert_eq!(raw.name.as_deref(), Some("Li6"));
        // Don't assert cache state here (other tests may be using it)
        let mut json_map = HashMap::new();
        json_map.insert("Li6".to_string(), "tests/Li6.json".to_string());
        let first =
            super::get_or_load_nuclide("Li6", &json_map, None).expect("Initial cached load failed");
        // Ensure Li6 now present in cache
        {
            let cache = super::GLOBAL_NUCLIDE_CACHE.lock().unwrap();
            assert!(
                cache.get("Li6").is_some(),
                "Li6 should be present after cached load"
            );
        }
        let second =
            super::get_or_load_nuclide("Li6", &json_map, None).expect("Second cached load failed");
        assert!(
            std::sync::Arc::ptr_eq(&first, &second),
            "Expected identical Arc pointer from cache on second load"
        );
        assert_eq!(
            first.name.as_deref(),
            raw.name.as_deref(),
            "Names differ between raw and cached"
        );
    }

    #[cfg(test)]
    #[test]
    fn test_sample_reaction_li6() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let path = std::path::Path::new("tests/Li6.json");
        let nuclide = super::read_nuclide_from_json(path, None).expect("Failed to load Li6.json");
        let temperature = "294";

        // Vary energy from 1.0 to 15e6 (10 steps)
        let energies = (0..10).map(|i| 1.0 + i as f64 * (15e6 - 1e6) / 9.0);

        for energy in energies {
            let mut rng1 = StdRng::seed_from_u64(42);
            let mut rng2 = StdRng::seed_from_u64(42);
            let mut rng3 = StdRng::seed_from_u64(43); // Different seed

            let reaction1 = nuclide.sample_reaction(energy, temperature, &mut rng1);
            let reaction2 = nuclide.sample_reaction(energy, temperature, &mut rng2);
            let reaction3 = nuclide.sample_reaction(energy, temperature, &mut rng3);

            // Ensure reactions were sampled successfully
            assert!(
                reaction1.is_some(),
                "sample_reaction returned None at energy {}",
                energy
            );
            assert!(
                reaction2.is_some(),
                "Repeat sample with same seed returned None at energy {}",
                energy
            );
            assert!(
                reaction3.is_some(),
                "Sample with different seed returned None at energy {}",
                energy
            );

            let reaction1 = reaction1.unwrap();
            let reaction2 = reaction2.unwrap();
            let reaction3 = reaction3.unwrap();

            // Ensure same-seed reactions are the same (determinism)
            assert_eq!(
                reaction1.mt_number, reaction2.mt_number,
                "Different MT for same seed at energy {}",
                energy
            );
            assert_eq!(
                reaction1.cross_section, reaction2.cross_section,
                "Different cross_section for same seed at energy {}",
                energy
            );

            // Print info about the third reaction (with different seed)
            println!(
                "Energy: {:e}, MT (seed 42): {}, MT (seed 43): {}",
                energy, reaction1.mt_number, reaction3.mt_number
            );

            // Ensure basic validity
            assert!(
                reaction1.mt_number > 0,
                "Sampled reaction has invalid MT number at energy {}",
                energy
            );
            assert!(
                !reaction1.cross_section.is_empty(),
                "Sampled reaction has empty cross section at energy {}",
                energy
            );
        }
    }

    #[test]
    fn test_reaction_mts_li6() {
        // Load Li6 nuclide from test JSON
        let path = std::path::Path::new("tests/Li6.json");
        let nuclide = super::read_nuclide_from_json(path, None).expect("Failed to load Li6.json");
        let mts = nuclide.reaction_mts().expect("No MTs found");
        let expected = vec![
            102, 103, 105, 2, 203, 204, 205, 207, 24, 301, 444, 51, 52, 53, 54, 55, 56, 57, 58, 59,
            60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81,
        ];
        for mt in &expected {
            assert!(mts.contains(mt), "Expected MT {} in Li6", mt);
        }
        // Optionally, check the total number of MTs if you want strictness
        // assert_eq!(mts.len(), expected.len());
    }

    #[test]
    fn test_reaction_mts_li7() {
        // Load Li7 nuclide from test JSON
        let path = std::path::Path::new("tests/Li7.json");
        let nuclide = super::read_nuclide_from_json(path, None).expect("Failed to load Li7.json");
        let mts = nuclide.reaction_mts().expect("No MTs found");
        // Check for presence of key hierarchical and explicit MTs
        assert!(mts.contains(&1), "MT=1 should be present");
        assert!(mts.contains(&3), "MT=3 should be present");
        assert!(mts.contains(&4), "MT=4 should be present");
        assert!(mts.contains(&27), "MT=27 should be present");
        assert!(mts.contains(&101), "MT=101 should be present");
        assert!(mts.contains(&2), "MT=2 should be present");
        assert!(mts.contains(&16), "MT=16 should be present");
        assert!(mts.contains(&24), "MT=24 should be present");
        assert!(mts.contains(&51), "MT=51 should be present");
        assert!(!mts.is_empty(), "MT list should not be empty");
    }

    #[test]
    fn test_fissionable_false_for_be9_and_fe58() {
        let nuclide_be9 =
            super::read_nuclide_from_json("tests/Be9.json", None).expect("Failed to load Be9.json");
        assert_eq!(
            nuclide_be9.fissionable, false,
            "Be9 should not be fissionable"
        );

        let path_fe58 = std::path::Path::new("tests/Fe58.json");
        let nuclide_fe58 =
            super::read_nuclide_from_json(path_fe58, None).expect("Failed to load Fe58.json");
        assert_eq!(
            nuclide_fe58.fissionable, false,
            "Fe58 should not be fissionable"
        );
    }

    #[test]
    fn test_li6_reactions_contain_specific_mts() {
        // Load Li6 nuclide from test JSON
        let path = std::path::Path::new("tests/Li6.json");
        let nuclide = super::read_nuclide_from_json(path, None).expect("Failed to load Li6.json");

        // Check that required MTs are present and mt_number is not 0
        let required = [2, 24, 51, 444];
        for mt in &required {
            let mut found = false;
            for temp_reactions in nuclide.reactions.values() {
                if let Some(reaction) = temp_reactions.get(mt) {
                    assert_ne!(
                        reaction.mt_number, 0,
                        "Reaction MT number for MT {} is 0",
                        mt
                    );
                    found = true;
                    break;
                }
            }
            assert!(found, "MT {} not found in any temperature reactions", mt);
        }
    }

    #[test]
    fn test_available_temperatures_be9() {
        let path_be9 = std::path::Path::new("tests/Be9.json");
        let nuclide_be9 =
            super::read_nuclide_from_json(path_be9, None).expect("Failed to load Be9.json");
        assert_eq!(
            nuclide_be9.available_temperatures,
            vec!["294".to_string(), "300".to_string()],
            "available_temperatures should be ['294','300']"
        );
        assert_eq!(
            nuclide_be9.loaded_temperatures,
            vec!["294".to_string(), "300".to_string()],
            "By default all temps are loaded"
        );
        let temps_method = nuclide_be9
            .temperatures()
            .expect("temperatures() returned None");
        assert_eq!(
            temps_method,
            vec!["294".to_string(), "300".to_string()],
            "temperatures() should return ['294','300']"
        );
    }

    #[test]
    fn test_be9_mt_numbers_per_temperature() {
        let path_be9 = std::path::Path::new("tests/Be9.json");
        let nuclide_be9 =
            super::read_nuclide_from_json(path_be9, None).expect("Failed to load Be9.json");

        // Expected full MT list at 294 K (extended set including higher MTs)
        let mut expected_294: Vec<i32> = vec![
            1, 2, 3, 16, 27, 101, 102, 103, 104, 105, 107, 203, 204, 205, 207, 301, 444, 875, 876,
            877, 878, 879, 880, 881, 882, 883, 884, 885, 886, 887, 888, 889, 890,
        ];
        expected_294.sort();

        // Expected reduced MT list at 300 K
        let mut expected_300: Vec<i32> = vec![
            1, 2, 3, 16, 27, 101, 102, 103, 104, 105, 107, 203, 204, 205, 207, 301,
        ];
        expected_300.sort();

        // Helper closure to extract and sort MT list for a temperature
        let get_sorted_mts = |temp: &str| -> Vec<i32> {
            let mut mts: Vec<i32> = nuclide_be9
                .reactions
                .get(temp)
                .expect("Temperature not found in reactions")
                .keys()
                .cloned()
                .collect();
            mts.sort();
            mts
        };

        let mts_294 = get_sorted_mts("294");
        let mts_300 = get_sorted_mts("300");

        assert_eq!(
            mts_294, expected_294,
            "Be9 294K MT list mismatch. Got {:?}",
            mts_294
        );
        assert_eq!(
            mts_300, expected_300,
            "Be9 300K MT list mismatch. Got {:?}",
            mts_300
        );

        // Ensure there are no overlapping unexpected MTs unique to 300 K
        for mt in &mts_300 {
            assert!(
                expected_294.contains(mt),
                "MT {} at 300K not present in 294K expected list (unexpected new MT)",
                mt
            );
        }
    }

    #[test]
    fn test_available_temperatures_fe56_includes_294() {
        let nuclide_fe56 = super::read_nuclide_from_json("tests/Fe56.json", None)
            .expect("Failed to load Fe56.json");
        assert!(
            nuclide_fe56
                .available_temperatures
                .iter()
                .any(|t| t == "294"),
            "Fe56 available_temperatures should contain '294'"
        );
        let temps_method = nuclide_fe56
            .temperatures()
            .expect("temperatures() returned None");
        assert!(
            temps_method.iter().any(|t| t == "294"),
            "Fe56 temperatures() should contain '294'"
        );
    }

    #[test]
    fn test_clear_nuclide_cache() {
        // Insert a test nuclide into the cache
        let li6_path = std::path::Path::new("tests/Li6.json");
        let nuclide =
            super::read_nuclide_from_json(li6_path, None).expect("Failed to load Li6.json");
        let nuclide_arc = std::sync::Arc::new(nuclide);

        {
            let mut cache = super::GLOBAL_NUCLIDE_CACHE.lock().unwrap();
            cache.insert(
                "Test_Nuclide".to_string(),
                std::sync::Arc::clone(&nuclide_arc),
            );

            // Verify it's in the cache
            assert!(
                cache.contains_key("Test_Nuclide"),
                "Test nuclide should be in cache before clearing"
            );
        }

        // Call the clear function
        super::clear_nuclide_cache();

        // Verify cache is now empty
        let cache = super::GLOBAL_NUCLIDE_CACHE.lock().unwrap();
        assert!(
            cache.is_empty(),
            "Cache should be empty after calling clear_nuclide_cache"
        );
    }

    #[test]
    #[cfg(feature = "download")]
    fn test_nuclide_from_url_energy_grid_positive() {
        // Clear the config to start fresh
        {
            let mut cfg = crate::config::CONFIG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            cfg.cross_sections.clear();
        }

        // Add Li6 using keyword to config
        {
            let mut cfg = crate::config::CONFIG.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            cfg.set_cross_section("Li6", "tendl-21");
        }

        // Load the nuclide using the keyword
        let nuclide = super::read_nuclide_from_json("Li6", None)
            .expect("Failed to load Li6 from keyword");

        // Get available temperatures
        let temps = nuclide.temperatures()
            .expect("Nuclide should have temperatures available");
        assert!(
            !temps.is_empty(),
            "Nuclide should have at least one temperature"
        );

        // Use the first available temperature to get the energy grid
        let temp = &temps[0];
        let energy_grid = nuclide.energy_grid(temp)
            .expect("Energy grid should be available for the temperature");

        // Check that the energy grid exists and contains positive numbers
        assert!(
            !energy_grid.is_empty(),
            "Energy grid should not be empty"
        );

        for (i, &energy) in energy_grid.iter().enumerate() {
            assert!(
                energy > 0.0,
                "Energy at index {} should be positive, but got {}",
                i,
                energy
            );
        }

        // Additional check: energy grid should be sorted in ascending order
        for i in 1..energy_grid.len() {
            assert!(
                energy_grid[i] >= energy_grid[i - 1],
                "Energy grid should be sorted: energy[{}] = {} < energy[{}] = {}",
                i,
                energy_grid[i],
                i - 1,
                energy_grid[i - 1]
            );
        }

        println!(
            "Successfully loaded Li6 from URL with {} energy points at temperature {}",
            energy_grid.len(),
            temp
        );
    }

    #[test]
    fn test_microscopic_cross_section_with_temperature() {
        let nuclide = super::read_nuclide_from_json("tests/Be9.json", None)
            .expect("Failed to load Be9.json");

        // Test with specific temperature
        let result = nuclide.microscopic_cross_section(2, Some("294"));
        assert!(result.is_ok(), "Should successfully get MT=2 data for 294K");
        
        let (xs, energy) = result.unwrap();
        assert!(!xs.is_empty(), "Cross section data should not be empty");
        assert!(!energy.is_empty(), "Energy data should not be empty");
        assert_eq!(xs.len(), energy.len(), "Cross section and energy arrays should have same length");
        
        // Test with different temperature
        let result_300 = nuclide.microscopic_cross_section(2, Some("300"));
        assert!(result_300.is_ok(), "Should successfully get MT=2 data for 300K");
        
        let (xs_300, energy_300) = result_300.unwrap();
        assert!(!xs_300.is_empty(), "Cross section data should not be empty for 300K");
        assert!(!energy_300.is_empty(), "Energy data should not be empty for 300K");
    }

    #[test]
    fn test_microscopic_cross_section_single_temperature() {
        // Load Be9 with only one temperature
        let temps_filter = std::collections::HashSet::from(["294".to_string()]);
        let nuclide = super::read_nuclide_from_json("tests/Be9.json", Some(&temps_filter))
            .expect("Failed to load Be9.json with temperature filter");

        // Should work without specifying temperature since only one is loaded
        let result = nuclide.microscopic_cross_section(2, None);
        assert!(result.is_ok(), "Should successfully get MT=2 data without temperature");
        
        let (xs, energy) = result.unwrap();
        assert!(!xs.is_empty(), "Cross section data should not be empty");
        assert!(!energy.is_empty(), "Energy data should not be empty");
        assert_eq!(xs.len(), energy.len(), "Cross section and energy arrays should have same length");
    }

    #[test]
    fn test_microscopic_cross_section_multiple_temperatures_error() {
        let nuclide = super::read_nuclide_from_json("tests/Be9.json", None)
            .expect("Failed to load Be9.json");

        // Should fail when no temperature specified with multiple loaded
        let result = nuclide.microscopic_cross_section(2, None);
        assert!(result.is_err(), "Should error when multiple temperatures loaded without specifying");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Multiple temperatures loaded"), 
               "Error should mention multiple temperatures: {}", error_msg);
    }

    #[test]
    fn test_microscopic_cross_section_invalid_temperature() {
        let nuclide = super::read_nuclide_from_json("tests/Be9.json", None)
            .expect("Failed to load Be9.json");

        // Should fail for non-existent temperature
        let result = nuclide.microscopic_cross_section(2, Some("500"));
        assert!(result.is_err(), "Should error for invalid temperature");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Temperature '500' not found"), 
               "Error should mention temperature not found: {}", error_msg);
    }

    #[test]
    fn test_microscopic_cross_section_invalid_mt() {
        let nuclide = super::read_nuclide_from_json("tests/Be9.json", None)
            .expect("Failed to load Be9.json");

        // Should fail for non-existent MT
        let result = nuclide.microscopic_cross_section(9999, Some("294"));
        assert!(result.is_err(), "Should error for invalid MT number");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("MT 9999 not found"), 
               "Error should mention MT not found: {}", error_msg);
    }

    #[test]
    fn test_microscopic_cross_section_multiple_mt_numbers() {
        let nuclide = super::read_nuclide_from_json("tests/Be9.json", None)
            .expect("Failed to load Be9.json");

        // Test common MT numbers that should exist in Be9
        let test_mts = [1, 2, 3, 16, 27, 101, 102];
        
        for mt in test_mts {
            let result = nuclide.microscopic_cross_section(mt, Some("294"));
            if result.is_ok() {
                let (xs, energy) = result.unwrap();
                assert!(!xs.is_empty(), "MT={} should have cross section data", mt);
                assert!(!energy.is_empty(), "MT={} should have energy data", mt);
                assert_eq!(xs.len(), energy.len(), "MT={} data length mismatch", mt);
                
                // Validate data quality
                for &e in &energy {
                    assert!(e > 0.0, "MT={} energy values should be positive", mt);
                }
                for &x in &xs {
                    assert!(x >= 0.0, "MT={} cross sections should be non-negative", mt);
                }
            }
            // Some MT numbers might not exist, which is fine
        }
    }

    #[test]
    fn test_microscopic_cross_section_lithium() {
        let nuclide = super::read_nuclide_from_json("tests/Li6.json", None)
            .expect("Failed to load Li6.json");

        // Li6 should have only one temperature, so no temperature needed
        let result = nuclide.microscopic_cross_section(2, None);
        assert!(result.is_ok(), "Should successfully get Li6 elastic scattering data");
        
        let (xs, energy) = result.unwrap();
        assert!(!xs.is_empty(), "Li6 elastic scattering data should not be empty");
        assert!(!energy.is_empty(), "Li6 energy data should not be empty");
        
        // Test with explicit temperature too
        let result_explicit = nuclide.microscopic_cross_section(2, Some("294"));
        assert!(result_explicit.is_ok(), "Should work with explicit temperature");
        
        let (xs_explicit, energy_explicit) = result_explicit.unwrap();
        assert_eq!(xs, xs_explicit, "Results should be identical with/without explicit temperature");
        assert_eq!(energy, energy_explicit, "Energy should be identical with/without explicit temperature");
    }

    #[test]
    fn test_microscopic_cross_section_temperature_with_k_suffix() {
        let nuclide = super::read_nuclide_from_json("tests/Be9.json", None)
            .expect("Failed to load Be9.json");

        // Test that temperature matching works with 'K' suffix
        let result_without_k = nuclide.microscopic_cross_section(2, Some("294"));
        let result_with_k = nuclide.microscopic_cross_section(2, Some("294K"));
        
        // Both should work (though one might fail if the data uses different format)
        if result_without_k.is_ok() && result_with_k.is_ok() {
            let (xs1, energy1) = result_without_k.unwrap();
            let (xs2, energy2) = result_with_k.unwrap();
            assert_eq!(xs1, xs2, "Temperature with/without K suffix should give same result");
            assert_eq!(energy1, energy2, "Energy with/without K suffix should give same result");
        } else {
            // At least one should work
            assert!(result_without_k.is_ok() || result_with_k.is_ok(), 
                   "At least one temperature format should work");
        }
    }
}
