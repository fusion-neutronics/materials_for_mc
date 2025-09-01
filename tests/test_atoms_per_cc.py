import pytest
from materials_for_mc import Material


def test_get_atoms_per_barn_cm():


    # Test with Li isotopes - proper atomic mass calculation
    material = Material()
    material.add_nuclide("Li6", 0.5)
    material.add_nuclide("Li7", 0.5)
    material.set_density("g/cm3", 1.0)
    
    atoms = material.get_atoms_per_barn_cm()
    assert len(atoms) == 2, "Should have 2 nuclides in the dict"
    
    # Calculate expected values using OpenMC's formula: N_A * density * fraction / atomic_mass
    avogadro = 6.02214076e23  # Atoms per mol
    li6_mass = 6.015122  # g/mol
    li7_mass = 7.016004  # g/mol
    
    # Average molar mass
    avg_molar_mass = (li6_mass * 0.5 + li7_mass * 0.5) / 1.0
    
    # The formula in the Rust code includes a factor of 1e-24 to convert to atoms/barn-cm
    # atoms/barn-cm = density * N_A / avg_molar_mass * normalized_fraction * 1e-24
    li6_expected = avogadro * 1.0 / avg_molar_mass * 0.5 * 1.0e-24  # atoms/barn-cm
    li7_expected = avogadro * 1.0 / avg_molar_mass * 0.5 * 1.0e-24  # atoms/barn-cm
    
    # Test with relative tolerance of 1%
    assert atoms["Li6"] == pytest.approx(li6_expected, rel=0.01), "Li6 atoms/cc calculation is incorrect"
    assert atoms["Li7"] == pytest.approx(li7_expected, rel=0.01), "Li7 atoms/cc calculation is incorrect"
    
    # Test with nuclides that don't have defined atomic masses
    # TODO check CustomNuclide results in panic
    # material = Material()
    # material.add_nuclide("CustomNuclide", 1.0)
    # material.set_density("g/cm3", 5.0)
    
    atoms = material.get_atoms_per_barn_cm()
    assert len(atoms) == 2, "Should have 2 nuclides in the dict"

 
    # Test with different density units (kg/m³)
    material = Material()
    material.add_nuclide("Li6", 1.0)
    material.set_density("kg/m3", 1000.0)  # 1000 kg/m³ = 1 g/cm³
    
    atoms_kg_m3 = material.get_atoms_per_barn_cm()
    assert len(atoms_kg_m3) == 1, "Should have 1 nuclide in the dict"
    
    # Compare with same material using g/cm³
    material_g_cm3 = Material()
    material_g_cm3.add_nuclide("Li6", 1.0)
    material_g_cm3.set_density("g/cm3", 1.0)
    
    atoms_g_cm3 = material_g_cm3.get_atoms_per_barn_cm()
    
    # Both should give the same result since the densities are equivalent
    assert atoms_kg_m3["Li6"] == pytest.approx(atoms_g_cm3["Li6"], rel=0.01), "Different density units should give consistent results"
