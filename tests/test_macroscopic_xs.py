import pytest
import numpy as np
from materials_for_mc import Material

def test_macroscopic_xs_neutron():
    # Create a material with Li6 and Li7
    material = Material()
    material.add_nuclide("Li6", 0.5)
    material.add_nuclide("Li7", 0.5)
    material.set_density("g/cm3", 1.0)  # Setting a simple density
    material.temperature = "294"  # Set temperature using the property
    material.read_nuclides_from_json({"Li6": "tests/li6_neutron.json", "Li7": "tests/li7_neutron.json"})
    
    # Get the unified energy grid - this will cache it
    grid = material.unified_energy_grid_neutron()
    
    # Calculate microscopic cross sections for all MT numbers using the cached grid
    micro_xs = material.calculate_microscopic_xs_neutron()
    
    # Verify that we have cross sections for both nuclides
    assert "Li6" in micro_xs, "No microscopic cross sections for Li6"
    assert "Li7" in micro_xs, "No microscopic cross sections for Li7"
    
    # Calculate macroscopic cross sections - will use the cached grid
    macro_xs = material.calculate_macroscopic_xs_neutron()
    
    # Verify the macroscopic cross sections contain MT=2
    assert "2" in macro_xs, "No MT=2 in macroscopic cross sections"
    
    # Verify the length of the macroscopic cross section array
    assert len(macro_xs["2"]) == len(grid), "Macro XS length doesn't match grid length"
    
    # Verify all values are non-negative
    assert all(xs >= 0 for xs in macro_xs["2"]), "Negative cross section values found"
    
    # Check that the macroscopic_xs_neutron property is accessible and contains data
    assert len(material.macroscopic_xs_neutron) > 0, "macroscopic_xs_neutron property is empty"
    assert material.macroscopic_xs_neutron == macro_xs, "macroscopic_xs_neutron doesn't match the calculated values"

def test_macroscopic_xs_with_atoms_per_cc():
    # Create a material with Li isotopes that have defined atomic masses
    material = Material()
    material.add_nuclide("Li6", 0.5)
    material.add_nuclide("Li7", 0.5)
    material.set_density("g/cm3", 1.0)
    material.temperature = "294"
    material.read_nuclides_from_json({"Li6": "tests/li6_neutron.json", "Li7": "tests/li7_neutron.json"})
    
    # Get the atoms per cc
    atoms_per_cc = material.get_atoms_per_cc()
    
    # Calculate macroscopic cross sections
    macro_xs = material.calculate_macroscopic_xs_neutron()
    
    # Verify that macroscopic cross sections were calculated
    assert len(macro_xs) > 0, "No macroscopic cross sections were calculated"
    
    # Test the relationship between density and macroscopic XS
    # If we double the density, atoms per cc should double, and so should macroscopic XS
    material.set_density("g/cm3", 2.0)
    atoms_per_cc_doubled = material.get_atoms_per_cc()
    macro_xs_doubled = material.calculate_macroscopic_xs_neutron()
    
    # Check that atoms per cc doubled
    for nuclide in atoms_per_cc.keys():
        assert atoms_per_cc_doubled[nuclide] == pytest.approx(2 * atoms_per_cc[nuclide], rel=1e-6)
    
    # Check that macroscopic XS doubled for each reaction
    for mt in macro_xs.keys():
        if mt in macro_xs_doubled:
            for i in range(len(macro_xs[mt])):
                # Only check if the original value is not zero to avoid division by zero
                if abs(macro_xs[mt][i]) > 1e-10:
                    assert macro_xs_doubled[mt][i] == pytest.approx(2 * macro_xs[mt][i], rel=1e-6)

def test_macroscopic_xs_calculation_formula():
    # Create a test material with nuclides that have defined atomic masses
    material = Material()
    material.add_nuclide("Li6", 1.0)  # Using a single nuclide for simplicity
    material.set_density("g/cm3", 1.0)
    material.temperature = "294"
    material.read_nuclides_from_json({"Li6": "tests/li6_neutron.json"})
    
    # Get microscopic cross sections
    micro_xs = material.calculate_microscopic_xs_neutron()
    
    # Get atoms per cc
    atoms_per_cc = material.get_atoms_per_cc()
    
    # Calculate macroscopic cross sections
    macro_xs = material.calculate_macroscopic_xs_neutron()
    
    # Verify that macroscopic XS = atoms_per_cc * microscopic_xs * BARN_TO_CM2
    BARN_TO_CM2 = 1.0e-24
    
    # Check for MT=2 (elastic scattering) if it exists
    if "2" in macro_xs and "Li6" in micro_xs and "2" in micro_xs["Li6"]:
        for i in range(min(10, len(macro_xs["2"]))):  # Check first 10 points or all if less
            expected = atoms_per_cc["Li6"] * micro_xs["Li6"]["2"][i] * BARN_TO_CM2
            assert macro_xs["2"][i] == pytest.approx(expected, rel=1e-6), \
                f"Macroscopic XS calculation incorrect at index {i}"
