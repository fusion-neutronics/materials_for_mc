use materials_for_mc::{Material, Config};


fn main() {

    Config::global().set_cross_section("Li6", "../../tests/Li6.json");
    Config::global().set_cross_section("Li7", "../../tests/Li7.json");

    let mut mat = Material::new();
    mat.add_nuclide("Li6", 0.05).unwrap();
    mat.set_density("g/cm3", 19.1).unwrap();
    mat.volume(Some(100.0)).unwrap();
    // mat.calculate_microscopic_xs_neutron(Some(&vec!["2".to_string()]));
    mat.calculate_macroscopic_xs_neutron(Some(&vec![3]));
    // mat.calculate_microscopic_xs_neutron(None);
    // mat.calculate_macroscopic_xs_neutron(None);

    println!("{:?}", mat);

    // Create a second material and add element Li (lithium) with natural abundances
    let mut lithium_mat = Material::new();
    lithium_mat.add_element("Li", 1.0).unwrap();
    lithium_mat.set_density("g/cm3", 0.534).unwrap(); // Lithium density
    lithium_mat.volume(Some(50.0)).unwrap();
    // lithium_mat.calculate_microscopic_xs_neutron(Some(&vec!["2".to_string()]));
    // lithium_mat.calculate_macroscopic_xs_neutron(Some(&vec!["2".to_string()]));
    // lithium_mat.calculate_microscopic_xs_neutron(None);
    lithium_mat.calculate_macroscopic_xs_neutron(None);

    // println!("Lithium material: {:?}", lithium_mat);
    // println!("Lithium material: {:?}", lithium_mat);
}