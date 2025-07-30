use materials_for_mc::{Material, Config};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {

    Config::global().set_cross_section("Li6", "../../tests/Li6.json");
    Config::global().set_cross_section("Li7", "../../tests/Li7.json");

    let mut mat = Material::new();
    mat.add_nuclide("Li6", 0.05).unwrap();
    mat.set_density("g/cm3", 19.1).unwrap();
    mat.volume(Some(100.0)).unwrap();
    // mat.calculate_microscopic_xs_neutron(Some(&vec!["2".to_string()]));
    mat.calculate_macroscopic_xs_neutron(&vec![3], false);
    // mat.calculate_microscopic_xs_neutron(None);
    // mat.calculate_macroscopic_xs_neutron(None);

    // println!("{:?}", mat);

    // Create a second material and add element Li (lithium) with natural abundances
    let mut lithium_mat = Material::new();
    lithium_mat.add_element("Li", 1.0).unwrap();
    lithium_mat.set_density("g/cm3", 0.534).unwrap(); // Lithium density
    lithium_mat.volume(Some(50.0)).unwrap();
    // lithium_mat.calculate_microscopic_xs_neutron(Some(&vec!["2".to_string()]));
    // lithium_mat.calculate_macroscopic_xs_neutron(Some(&vec!["2".to_string()]));
    // lithium_mat.calculate_microscopic_xs_neutron(None);
    // Print available MT numbers for each nuclide at temperature "294"
    for (nuclide, nuclide_data) in &lithium_mat.nuclide_data {
        if let Some(reactions) = nuclide_data.reactions.get("294") {
            let mt_keys: Vec<_> = reactions.keys().collect();
            println!("[DEBUG] Nuclide {} MT keys at 294: {:?}", nuclide, mt_keys);
        } else {
            println!("[DEBUG] Nuclide {} has no reactions for temperature 294", nuclide);
        }
    }
    lithium_mat.calculate_macroscopic_xs_neutron(&vec![1], true);


    let mut rng = StdRng::seed_from_u64(123456);
    let energy = 1.0e3; // 1 MeV
    // Sample nuclide
    let sampled_nuclide_name = lithium_mat.sample_interacting_nuclide(energy, &mut rng);
    println!("Sampled nuclide: {}", sampled_nuclide_name);
    if let Some(nuclide) = lithium_mat.nuclide_data.get(&sampled_nuclide_name) {
        println!("fissionable: {}", nuclide.fissionable);
    } else {
        println!("Nuclide struct not found for {}", sampled_nuclide_name);
    }

    // Print the per-nuclide macroscopic total cross sections for lithium_mat
    // println!("lithium_mat.macroscopic_total_xs_by_nuclide: {:#?}", lithium_mat.macroscopic_total_xs_by_nuclide);
}