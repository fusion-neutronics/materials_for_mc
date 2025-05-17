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
}