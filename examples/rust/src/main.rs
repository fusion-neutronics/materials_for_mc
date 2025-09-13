use materials_for_mc::{Material, Config};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {

    Config::global().set_cross_section("Fe56", "../../tests/Fe56.json");
    let cross_sections = std::collections::HashMap::from([
        ("Li7".to_string(), "../../tests/Li7.json".to_string()),
        ("Li6".to_string(), "tendl-21".to_string()),
    ]);
    Config::global().set_cross_sections(cross_sections);

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
    lithium_mat.add_nuclide("Fe56", 1.0).unwrap();
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

    // ------------------------------------------------------------
    // Performance timing: mean free path sampling across energies
    // ------------------------------------------------------------
    use std::time::Instant;
    let n_energies = 100; // number of distinct energies
    let n_samples_per_energy = 1000; // number of samples per energy
    // Build energies logarithmically spaced between 1e2 eV and 1e6 eV
    let e_min = 1.0e2_f64;
    let e_max = 1.0e6_f64;
    let log_min = e_min.ln();
    let log_max = e_max.ln();
    let mut energies: Vec<f64> = Vec::with_capacity(n_energies);
    for i in 0..n_energies {
        let f = i as f64 / (n_energies as f64 - 1.0);
        energies.push((log_min + f * (log_max - log_min)).exp());
    }
    // Ensure total xs grid is built once (MT=1)
    lithium_mat.calculate_macroscopic_xs_neutron(&vec![1], false);
    let start = Instant::now();
    let mut total_queries: u64 = 0;
    let mut accum_mfp = 0.0_f64; // accumulate to prevent optimization
    for &e in &energies {
        for _ in 0..n_samples_per_energy {
            if let Some(mfp) = lithium_mat.mean_free_path_neutron(e) {
                accum_mfp += mfp;
            }
            total_queries += 1;
        }
    }
    let elapsed = start.elapsed();
    let secs = elapsed.as_secs_f64();
    let per_call_ns = (secs * 1.0e9) / (total_queries as f64);
    println!(
        "Mean free path sampling: energies={} samples/energy={} total_calls={} total_time={:.6}s avg_per_call={:.1} ns (accum_mfp={:.3})",
        n_energies,
        n_samples_per_energy,
        total_queries,
        secs,
        per_call_ns,
        accum_mfp
    );
}