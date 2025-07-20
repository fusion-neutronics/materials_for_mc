// Provides functionality for working with natural elements and their isotopic abundances
use std::collections::HashMap;
use once_cell::sync::Lazy;

use crate::material::Material;

// Natural abundances of isotopes from IUPAC Technical Report (2013)
// Values represent atomic fractions
static NATURAL_ABUNDANCE: Lazy<HashMap<&'static str, f64>> = Lazy::new(|| {
    let mut abundances = HashMap::new();
    
    // Hydrogen and Helium
    abundances.insert("H1", 0.99984426);
    abundances.insert("H2", 0.00015574);
    abundances.insert("He3", 0.000002);
    abundances.insert("He4", 0.999998);
    
    // Lithium, Beryllium, Boron
    abundances.insert("Li6", 0.07589);
    abundances.insert("Li7", 0.92411);
    abundances.insert("Be9", 1.0);
    abundances.insert("B10", 0.1982);
    abundances.insert("B11", 0.8018);
    
    // Carbon, Nitrogen, Oxygen, Fluorine, Neon
    abundances.insert("C12", 0.988922);
    abundances.insert("C13", 0.011078);
    abundances.insert("N14", 0.996337);
    abundances.insert("N15", 0.003663);
    abundances.insert("O16", 0.9976206);
    abundances.insert("O17", 0.000379);
    abundances.insert("O18", 0.0020004);
    abundances.insert("F19", 1.0);
    abundances.insert("Ne20", 0.9048);
    abundances.insert("Ne21", 0.0027);
    abundances.insert("Ne22", 0.0925);
    
    // Sodium, Magnesium, Aluminum, Silicon, Phosphorus
    abundances.insert("Na23", 1.0);
    abundances.insert("Mg24", 0.78951);
    abundances.insert("Mg25", 0.1002);
    abundances.insert("Mg26", 0.11029);
    abundances.insert("Al27", 1.0);
    abundances.insert("Si28", 0.9222968);
    abundances.insert("Si29", 0.0468316);
    abundances.insert("Si30", 0.0308716);
    abundances.insert("P31", 1.0);
    
    // Sulfur, Chlorine, Argon, Potassium, Calcium
    abundances.insert("S32", 0.9504074);
    abundances.insert("S33", 0.0074869);
    abundances.insert("S34", 0.0419599);
    abundances.insert("S36", 0.0001458);
    abundances.insert("Cl35", 0.757647);
    abundances.insert("Cl37", 0.242353);
    abundances.insert("Ar36", 0.003336);
    abundances.insert("Ar38", 0.000629);
    abundances.insert("Ar40", 0.996035);
    abundances.insert("K39", 0.932581);
    abundances.insert("K40", 0.000117);
    abundances.insert("K41", 0.067302);
    abundances.insert("Ca40", 0.96941);
    abundances.insert("Ca42", 0.00647);
    abundances.insert("Ca43", 0.00135);
    abundances.insert("Ca44", 0.02086);
    abundances.insert("Ca46", 0.00004);
    abundances.insert("Ca48", 0.00187);
    
    // Scandium, Titanium, Vanadium, Chromium, Manganese
    abundances.insert("Sc45", 1.0);
    abundances.insert("Ti46", 0.0825);
    abundances.insert("Ti47", 0.0744);
    abundances.insert("Ti48", 0.7372);
    abundances.insert("Ti49", 0.0541);
    abundances.insert("Ti50", 0.0518);
    abundances.insert("V50", 0.0025);
    abundances.insert("V51", 0.9975);
    abundances.insert("Cr50", 0.04345);
    abundances.insert("Cr52", 0.83789);
    abundances.insert("Cr53", 0.09501);
    abundances.insert("Cr54", 0.02365);
    abundances.insert("Mn55", 1.0);
    
    // Iron, Cobalt, Nickel, Copper, Zinc
    abundances.insert("Fe54", 0.05845);
    abundances.insert("Fe56", 0.91754);
    abundances.insert("Fe57", 0.02119);
    abundances.insert("Fe58", 0.00282);
    abundances.insert("Co59", 1.0);
    abundances.insert("Ni58", 0.680769);
    abundances.insert("Ni60", 0.262231);
    abundances.insert("Ni61", 0.011399);
    abundances.insert("Ni62", 0.036345);
    abundances.insert("Ni64", 0.009256);
    abundances.insert("Cu63", 0.6915);
    abundances.insert("Cu65", 0.3085);
    abundances.insert("Zn64", 0.4917);
    abundances.insert("Zn66", 0.2773);
    abundances.insert("Zn67", 0.0404);
    abundances.insert("Zn68", 0.1845);
    abundances.insert("Zn70", 0.0061);
    
    // Gallium through Rubidium
    abundances.insert("Ga69", 0.60108);
    abundances.insert("Ga71", 0.39892);
    abundances.insert("Ge70", 0.2052);
    abundances.insert("Ge72", 0.2745);
    abundances.insert("Ge73", 0.0776);
    abundances.insert("Ge74", 0.3652);
    abundances.insert("Ge76", 0.0775);
    abundances.insert("As75", 1.0);
    abundances.insert("Se74", 0.0086);
    abundances.insert("Se76", 0.0923);
    abundances.insert("Se77", 0.076);
    abundances.insert("Se78", 0.2369);
    abundances.insert("Se80", 0.498);
    abundances.insert("Se82", 0.0882);
    abundances.insert("Br79", 0.50686);
    abundances.insert("Br81", 0.49314);
    abundances.insert("Kr78", 0.00355);
    abundances.insert("Kr80", 0.02286);
    abundances.insert("Kr82", 0.11593);
    abundances.insert("Kr83", 0.115);
    abundances.insert("Kr84", 0.56987);
    abundances.insert("Kr86", 0.17279);
    abundances.insert("Rb85", 0.7217);
    abundances.insert("Rb87", 0.2783);
    
    // Strontium through Molybdenum
    abundances.insert("Sr84", 0.0056);
    abundances.insert("Sr86", 0.0986);
    abundances.insert("Sr87", 0.07);
    abundances.insert("Sr88", 0.8258);
    abundances.insert("Y89", 1.0);
    abundances.insert("Zr90", 0.5145);
    abundances.insert("Zr91", 0.1122);
    abundances.insert("Zr92", 0.1715);
    abundances.insert("Zr94", 0.1738);
    abundances.insert("Zr96", 0.028);
    abundances.insert("Nb93", 1.0);
    abundances.insert("Mo92", 0.14649);
    abundances.insert("Mo94", 0.09187);
    abundances.insert("Mo95", 0.15873);
    abundances.insert("Mo96", 0.16673);
    abundances.insert("Mo97", 0.09582);
    abundances.insert("Mo98", 0.24292);
    abundances.insert("Mo100", 0.09744);
    
    // Ruthenium through Silver
    abundances.insert("Ru96", 0.0554);
    abundances.insert("Ru98", 0.0187);
    abundances.insert("Ru99", 0.1276);
    abundances.insert("Ru100", 0.126);
    abundances.insert("Ru101", 0.1706);
    abundances.insert("Ru102", 0.3155);
    abundances.insert("Ru104", 0.1862);
    abundances.insert("Rh103", 1.0);
    abundances.insert("Pd102", 0.0102);
    abundances.insert("Pd104", 0.1114);
    abundances.insert("Pd105", 0.2233);
    abundances.insert("Pd106", 0.2733);
    abundances.insert("Pd108", 0.2646);
    abundances.insert("Pd110", 0.1172);
    abundances.insert("Ag107", 0.51839);
    abundances.insert("Ag109", 0.48161);
    
    // Cadmium through Xenon
    abundances.insert("Cd106", 0.01245);
    abundances.insert("Cd108", 0.00888);
    abundances.insert("Cd110", 0.1247);
    abundances.insert("Cd111", 0.12795);
    abundances.insert("Cd112", 0.24109);
    abundances.insert("Cd113", 0.12227);
    abundances.insert("Cd114", 0.28754);
    abundances.insert("Cd116", 0.07512);
    abundances.insert("In113", 0.04281);
    abundances.insert("In115", 0.95719);
    abundances.insert("Sn112", 0.0097);
    abundances.insert("Sn114", 0.0066);
    abundances.insert("Sn115", 0.0034);
    abundances.insert("Sn116", 0.1454);
    abundances.insert("Sn117", 0.0768);
    abundances.insert("Sn118", 0.2422);
    abundances.insert("Sn119", 0.0859);
    abundances.insert("Sn120", 0.3258);
    abundances.insert("Sn122", 0.0463);
    abundances.insert("Sn124", 0.0579);
    abundances.insert("Sb121", 0.5721);
    abundances.insert("Sb123", 0.4279);
    abundances.insert("Te120", 0.0009);
    abundances.insert("Te122", 0.0255);
    abundances.insert("Te123", 0.0089);
    abundances.insert("Te124", 0.0474);
    abundances.insert("Te125", 0.0707);
    abundances.insert("Te126", 0.1884);
    abundances.insert("Te128", 0.3174);
    abundances.insert("Te130", 0.3408);
    abundances.insert("I127", 1.0);
    abundances.insert("Xe124", 0.00095);
    abundances.insert("Xe126", 0.00089);
    abundances.insert("Xe128", 0.0191);
    abundances.insert("Xe129", 0.26401);
    abundances.insert("Xe130", 0.04071);
    abundances.insert("Xe131", 0.21232);
    abundances.insert("Xe132", 0.26909);
    abundances.insert("Xe134", 0.10436);
    abundances.insert("Xe136", 0.08857);
    
    // Cesium through Gadolinium
    abundances.insert("Cs133", 1.0);
    abundances.insert("Ba130", 0.0011);
    abundances.insert("Ba132", 0.001);
    abundances.insert("Ba134", 0.0242);
    abundances.insert("Ba135", 0.0659);
    abundances.insert("Ba136", 0.0785);
    abundances.insert("Ba137", 0.1123);
    abundances.insert("Ba138", 0.717);
    abundances.insert("La138", 0.0008881);
    abundances.insert("La139", 0.9991119);
    abundances.insert("Ce136", 0.00186);
    abundances.insert("Ce138", 0.00251);
    abundances.insert("Ce140", 0.88449);
    abundances.insert("Ce142", 0.11114);
    abundances.insert("Pr141", 1.0);
    abundances.insert("Nd142", 0.27153);
    abundances.insert("Nd143", 0.12173);
    abundances.insert("Nd144", 0.23798);
    abundances.insert("Nd145", 0.08293);
    abundances.insert("Nd146", 0.17189);
    abundances.insert("Nd148", 0.05756);
    abundances.insert("Nd150", 0.05638);
    abundances.insert("Sm144", 0.0308);
    abundances.insert("Sm147", 0.15);
    abundances.insert("Sm148", 0.1125);
    abundances.insert("Sm149", 0.1382);
    abundances.insert("Sm150", 0.0737);
    abundances.insert("Sm152", 0.2674);
    abundances.insert("Sm154", 0.2274);
    abundances.insert("Eu151", 0.4781);
    abundances.insert("Eu153", 0.5219);
    abundances.insert("Gd152", 0.002);
    abundances.insert("Gd154", 0.0218);
    abundances.insert("Gd155", 0.148);
    abundances.insert("Gd156", 0.2047);
    abundances.insert("Gd157", 0.1565);
    abundances.insert("Gd158", 0.2484);
    abundances.insert("Gd160", 0.2186);
    
    // Terbium through Lutetium
    abundances.insert("Tb159", 1.0);
    abundances.insert("Dy156", 0.00056);
    abundances.insert("Dy158", 0.00095);
    abundances.insert("Dy160", 0.02329);
    abundances.insert("Dy161", 0.18889);
    abundances.insert("Dy162", 0.25475);
    abundances.insert("Dy163", 0.24896);
    abundances.insert("Dy164", 0.2826);
    abundances.insert("Ho165", 1.0);
    abundances.insert("Er162", 0.00139);
    abundances.insert("Er164", 0.01601);
    abundances.insert("Er166", 0.33503);
    abundances.insert("Er167", 0.22869);
    abundances.insert("Er168", 0.26978);
    abundances.insert("Er170", 0.1491);
    abundances.insert("Tm169", 1.0);
    abundances.insert("Yb168", 0.00123);
    abundances.insert("Yb170", 0.02982);
    abundances.insert("Yb171", 0.14086);
    abundances.insert("Yb172", 0.21686);
    abundances.insert("Yb173", 0.16103);
    abundances.insert("Yb174", 0.32025);
    abundances.insert("Yb176", 0.12995);
    abundances.insert("Lu175", 0.97401);
    abundances.insert("Lu176", 0.02599);
    
    // Hafnium through Mercury
    abundances.insert("Hf174", 0.0016);
    abundances.insert("Hf176", 0.0526);
    abundances.insert("Hf177", 0.186);
    abundances.insert("Hf178", 0.2728);
    abundances.insert("Hf179", 0.1362);
    abundances.insert("Hf180", 0.3508);
    abundances.insert("Ta180_m1", 0.0001201);
    abundances.insert("Ta181", 0.9998799);
    abundances.insert("W180", 0.0012);
    abundances.insert("W182", 0.265);
    abundances.insert("W183", 0.1431);
    abundances.insert("W184", 0.3064);
    abundances.insert("W186", 0.2843);
    abundances.insert("Re185", 0.374);
    abundances.insert("Re187", 0.626);
    abundances.insert("Os184", 0.0002);
    abundances.insert("Os186", 0.0159);
    abundances.insert("Os187", 0.0196);
    abundances.insert("Os188", 0.1324);
    abundances.insert("Os189", 0.1615);
    abundances.insert("Os190", 0.2626);
    abundances.insert("Os192", 0.4078);
    abundances.insert("Ir191", 0.373);
    abundances.insert("Ir193", 0.627);
    abundances.insert("Pt190", 0.00012);
    abundances.insert("Pt192", 0.00782);
    abundances.insert("Pt194", 0.32864);
    abundances.insert("Pt195", 0.33775);
    abundances.insert("Pt196", 0.25211);
    abundances.insert("Pt198", 0.07356);
    abundances.insert("Au197", 1.0);
    abundances.insert("Hg196", 0.0015);
    abundances.insert("Hg198", 0.1004);
    abundances.insert("Hg199", 0.1694);
    abundances.insert("Hg200", 0.2314);
    abundances.insert("Hg201", 0.1317);
    abundances.insert("Hg202", 0.2974);
    abundances.insert("Hg204", 0.0682);
    
    // Thallium through Uranium
    abundances.insert("Tl203", 0.29524);
    abundances.insert("Tl205", 0.70476);
    abundances.insert("Pb204", 0.014);
    abundances.insert("Pb206", 0.241);
    abundances.insert("Pb207", 0.221);
    abundances.insert("Pb208", 0.524);
    abundances.insert("Bi209", 1.0);
    abundances.insert("Th230", 0.0002);
    abundances.insert("Th232", 0.9998);
    abundances.insert("Pa231", 1.0);
    abundances.insert("U234", 0.000054);
    abundances.insert("U235", 0.007204);
    abundances.insert("U238", 0.992742);
    
    abundances
});

// Map of element symbols to their full names
static ELEMENT_NAMES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut names = HashMap::new();
    names.insert("H", "hydrogen");
    names.insert("He", "helium");
    names.insert("Li", "lithium");
    names.insert("Be", "beryllium");
    names.insert("B", "boron");
    names.insert("C", "carbon");
    names.insert("N", "nitrogen");
    names.insert("O", "oxygen");
    names.insert("F", "fluorine");
    names.insert("Ne", "neon");
    names.insert("Na", "sodium");
    names.insert("Mg", "magnesium");
    names.insert("Al", "aluminum");
    names.insert("Si", "silicon");
    names.insert("P", "phosphorus");
    names.insert("S", "sulfur");
    names.insert("Cl", "chlorine");
    names.insert("Ar", "argon");
    names.insert("K", "potassium");
    names.insert("Ca", "calcium");
    names.insert("Sc", "scandium");
    names.insert("Ti", "titanium");
    names.insert("V", "vanadium");
    names.insert("Cr", "chromium");
    names.insert("Mn", "manganese");
    names.insert("Fe", "iron");
    names.insert("Co", "cobalt");
    names.insert("Ni", "nickel");
    names.insert("Cu", "copper");
    names.insert("Zn", "zinc");
    names.insert("Ga", "gallium");
    names.insert("Ge", "germanium");
    names.insert("As", "arsenic");
    names.insert("Se", "selenium");
    names.insert("Br", "bromine");
    names.insert("Kr", "krypton");
    names.insert("Rb", "rubidium");
    names.insert("Sr", "strontium");
    names.insert("Y", "yttrium");
    names.insert("Zr", "zirconium");
    names.insert("Nb", "niobium");
    names.insert("Mo", "molybdenum");
    names.insert("Tc", "technetium");
    names.insert("Ru", "ruthenium");
    names.insert("Rh", "rhodium");
    names.insert("Pd", "palladium");
    names.insert("Ag", "silver");
    names.insert("Cd", "cadmium");
    names.insert("In", "indium");
    names.insert("Sn", "tin");
    names.insert("Sb", "antimony");
    names.insert("Te", "tellurium");
    names.insert("I", "iodine");
    names.insert("Xe", "xenon");
    names.insert("Cs", "cesium");
    names.insert("Ba", "barium");
    names.insert("La", "lanthanum");
    names.insert("Ce", "cerium");
    names.insert("Pr", "praseodymium");
    names.insert("Nd", "neodymium");
    names.insert("Pm", "promethium");
    names.insert("Sm", "samarium");
    names.insert("Eu", "europium");
    names.insert("Gd", "gadolinium");
    names.insert("Tb", "terbium");
    names.insert("Dy", "dysprosium");
    names.insert("Ho", "holmium");
    names.insert("Er", "erbium");
    names.insert("Tm", "thulium");
    names.insert("Yb", "ytterbium");
    names.insert("Lu", "lutetium");
    names.insert("Hf", "hafnium");
    names.insert("Ta", "tantalum");
    names.insert("W", "tungsten");
    names.insert("Re", "rhenium");
    names.insert("Os", "osmium");
    names.insert("Ir", "iridium");
    names.insert("Pt", "platinum");
    names.insert("Au", "gold");
    names.insert("Hg", "mercury");
    names.insert("Tl", "thallium");
    names.insert("Pb", "lead");
    names.insert("Bi", "bismuth");
    names.insert("Po", "polonium");
    names.insert("At", "astatine");
    names.insert("Rn", "radon");
    names.insert("Fr", "francium");
    names.insert("Ra", "radium");
    names.insert("Ac", "actinium");
    names.insert("Th", "thorium");
    names.insert("Pa", "protactinium");
    names.insert("U", "uranium");
    names.insert("Np", "neptunium");
    names.insert("Pu", "plutonium");
    names.insert("Am", "americium");
    names.insert("Cm", "curium");
    names.insert("Bk", "berkelium");
    names.insert("Cf", "californium");
    names.insert("Es", "einsteinium");
    names.insert("Fm", "fermium");
    names.insert("Md", "mendelevium");
    names.insert("No", "nobelium");
    names.insert("Lr", "lawrencium");
    names
});

/// Mapping from element symbol to list of isotopes for that element
fn get_element_isotopes() -> HashMap<&'static str, Vec<&'static str>> {
    let mut element_isotopes: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
    
    // Build a mapping from elements to their isotopes
    for isotope in NATURAL_ABUNDANCE.keys() {
        // Extract the element symbol (all characters before the first digit)
        let mut i = 0;
        while i < isotope.len() && !isotope.chars().nth(i).unwrap().is_digit(10) {
            i += 1;
        }
        
        let element = &isotope[0..i];
        
        element_isotopes.entry(element)
            .or_insert_with(Vec::new)
            .push(isotope);
    }
    
    // Sort isotopes by mass number for each element
    for isotopes in element_isotopes.values_mut() {
        isotopes.sort_by(|a, b| {
            // Extract the mass number as a number
            let a_mass = a[0..].chars().filter(|c| c.is_digit(10))
                .collect::<String>().parse::<u32>().unwrap();
            let b_mass = b[0..].chars().filter(|c| c.is_digit(10))
                .collect::<String>().parse::<u32>().unwrap();
            a_mass.cmp(&b_mass)
        });
    }
    
    element_isotopes
}

/// Extension trait for Material to add element-related functionality
pub trait ElementExtensions {
    fn add_element(&mut self, element: &str, fraction: f64) -> Result<(), String>;
    
    /// Get the list of available elements
    fn get_available_elements() -> Vec<String>;
}

impl ElementExtensions for Material {
    fn add_element(&mut self, element: &str, fraction: f64) -> Result<(), String> {
        if fraction <= 0.0 {
            return Err(String::from("Fraction must be positive"));
        }
        
        // Get the element symbol in proper case (first letter uppercase, rest lowercase)
        let element_sym = if element.len() >= 2 {
            let mut e = element.to_lowercase();
            e.replace_range(0..1, &element[0..1].to_uppercase());
            e
        } else {
            element.to_uppercase()
        };
        
        // Get the isotopes for this element
        let element_isotopes = get_element_isotopes();
        
        // Check if the element exists in our database
        let isotopes = element_isotopes.get(element_sym.as_str()).ok_or_else(|| {
            format!("Element '{}' not found in the natural abundance database", element)
        })?;
        
        // Add each isotope with its natural abundance
        for &isotope in isotopes {
            let abundance = NATURAL_ABUNDANCE.get(isotope).unwrap();
            let isotope_fraction = fraction * abundance;
            
            // Only add isotopes with non-zero fractions
            if isotope_fraction > 0.0 {
                self.add_nuclide(isotope, isotope_fraction)?;
            }
        }
        
        Ok(())
    }
    
    /// Get the list of available elements
    fn get_available_elements() -> Vec<String> {
        let element_isotopes = get_element_isotopes();
        let mut elements: Vec<String> = element_isotopes.keys()
            .map(|&elem| elem.to_string())
            .collect();
        elements.sort();
        elements
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_element_with_natural_abundances() {
        let mut material = Material::new();
        
        // Test adding natural lithium
        let result = material.add_element_with_natural_abundances("Li", 1.0);
        assert!(result.is_ok());
        
        // Verify the isotopes were added correctly
        assert!(material.nuclides.contains_key("Li6"));
        assert!(material.nuclides.contains_key("Li7"));
        
        // Check the fractions are correct
        assert_eq!(*material.nuclides.get("Li6").unwrap(), 0.07589);
        assert_eq!(*material.nuclides.get("Li7").unwrap(), 0.92411);
        
        // Test adding an element with many isotopes
        let mut material2 = Material::new();
        let result = material2.add_element_with_natural_abundances("Sn", 1.0); // Tin has 10 isotopes
        assert!(result.is_ok());
        assert_eq!(material2.nuclides.len(), 10);
    }
    
    #[test]
    fn test_add_element_invalid() {
        let mut material = Material::new();
        
        // Test with negative fraction
        let result = material.add_element_with_natural_abundances("Li", -1.0);
        assert!(result.is_err());
        
        // Test with invalid element
        let result = material.add_element_with_natural_abundances("Xx", 1.0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_get_available_elements() {
        let elements = Material::get_available_elements();
        
        // Check that some common elements are in the list
        assert!(elements.contains(&"H".to_string()));
        assert!(elements.contains(&"He".to_string()));
        assert!(elements.contains(&"Li".to_string()));
        assert!(elements.contains(&"U".to_string()));
        
        // Check that the list is sorted
        let mut sorted_elements = elements.clone();
        sorted_elements.sort();
        assert_eq!(elements, sorted_elements);
    }
}
