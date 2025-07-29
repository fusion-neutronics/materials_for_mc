use materials_for_mc::{Material, Config};


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

    lithium_mat.calculate_macroscopic_xs_neutron(&vec![1], true);

    // Sample the nuclide of interaction and then sample the reaction MT
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    let mut rng = StdRng::seed_from_u64(123456);
    let energy = 1.0e3; // 1 MeV
    // Sample nuclide
    let sampled_nuclide = lithium_mat.sample_interacting_nuclide(energy, &mut rng);
    println!("Sampled nuclide: {}", sampled_nuclide);
    // Get the nuclide object
    let nuclide_obj = lithium_mat.nuclide_data.get(&sampled_nuclide).expect("Nuclide data not loaded");
    // Sample reaction MT
    let sampled_mt = nuclide_obj.sample_reaction_mt(energy, "294", &mut rng);
    println!("Sampled reaction MT: {}", sampled_mt);
}