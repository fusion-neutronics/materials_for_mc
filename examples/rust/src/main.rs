// Change this:
// use materials_for_mc::material::Material;

// To this:
use materials_for_mc::Material;


fn main() {
    let mut mat = Material::new();
    mat.add_nuclide("U235", 0.05).unwrap();
    mat.set_density("g/cm3", 19.1).unwrap();
    mat.volume(Some(100.0)).unwrap();

    println!("{:?}", mat);

    // Create a second material and add element Li (lithium) with natural abundances
    let mut lithium_mat = Material::new();
    lithium_mat.add_element("Li", 1.0).unwrap();
    lithium_mat.set_density("g/cm3", 0.534).unwrap(); // Lithium density
    lithium_mat.volume(Some(50.0)).unwrap();

    println!("Lithium material: {:?}", lithium_mat);
}