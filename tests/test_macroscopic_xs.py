import pytest
import numpy as np
from materials_for_mc import Material

def test_macroscopic_xs():
    # Create a material with Li6 and Li7
    material = Material()
    material.add_nuclide("Li6", 1.0)
    material.add_nuclide("Li7", 1.0)
    material.set_density("g/cm3", 1.0)  # Setting a simple density
    material.temperature = "294"  # Set temperature using the property
    material.read_nuclides_from_json({"Li6": "tests/li6.json", "Li7": "tests/li7.json"})
    
    # Get the unified energy grid - this will cache it
    grid = material.unified_energy_grid("neutron")
    
    # Calculate microscopic cross sections for all MT numbers using the cached grid
    micro_xs = material.calculate_microscopic_xs("neutron")
    
    # Verify that we have cross sections for both nuclides
    assert "Li6" in micro_xs, "No microscopic cross sections for Li6"
    assert "Li7" in micro_xs, "No microscopic cross sections for Li7"
    
    # Calculate macroscopic cross sections - will use the cached grid
    macro_xs = material.calculate_macroscopic_xs("neutron")
    
    # Verify the macroscopic cross sections contain MT=2
    assert "2" in macro_xs, "No MT=2 in macroscopic cross sections"
    
    # Verify the length of the macroscopic cross section array
    assert len(macro_xs["2"]) == len(grid), "Macro XS length doesn't match grid length"
    
    # Verify all values are non-negative
    assert all(xs >= 0 for xs in macro_xs["2"]), "Negative cross section values found"
    
    # Check that the macroscopic_xs_neutron property is accessible and contains data
    assert len(material.macroscopic_xs_neutron) > 0, "macroscopic_xs_neutron property is empty"
    assert material.macroscopic_xs_neutron == macro_xs, "macroscopic_xs_neutron doesn't match the calculated values"
    
    # # Verify all values are non-negative
    # assert all(xs >= 0 for xs in macro_xs["2"]), "Negative cross section values found"
    
    # # Verify that macroscopic XS is approximately the sum of microscopic XS times density
    # # This is a simplified check since we're using a simple atoms_per_cc calculation
    # BARN_TO_CM2 = 1.0e-24
    # density = 1.0  # We set it to 1.0 g/cm3
    
    # # Check a few random points for MT=2
    # for i in range(0, len(grid), max(1, len(grid) // 10)):  # Check ~10 points
    #     expected = (micro_xs["Li6"]["2"][i] + micro_xs["Li7"]["2"][i]) * density * BARN_TO_CM2
    #     assert abs(macro_xs["2"][i] - expected) < 1e-10, f"Macroscopic XS calculation incorrect at index {i}"
    
    # # Check that we have multiple MT numbers
    # assert len(macro_xs) > 1, "Expected multiple MT numbers in macroscopic cross sections"
    
    # # Test explicit grid parameter - calculate again with explicit grid
    # explicit_grid = grid[::2]  # Use every other point from the original grid
    # micro_xs_explicit = material.calculate_microscopic_xs("neutron")
    # assert len(micro_xs_explicit["Li6"]["2"]) == len(explicit_grid), "Explicit grid not used correctly"
