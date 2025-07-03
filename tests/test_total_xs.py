import pytest
from materials_for_mc import Material, Config

def test_calculate_total_xs_neutron():
    # Set up global Config
    Config.set_cross_sections({"Li6": "tests/li6_neutron.json", "Li7": "tests/li7_neutron.json"})
    
    # Create a material
    material = Material()
    material.add_nuclide("Li6", 1.0)
    material.add_nuclide("Li7", 1.0)
    material.set_density("g/cm3", 1.0)
    material.temperature = "294"
    
    # Calculate macroscopic cross sections
    macro_xs = material.calculate_macroscopic_xs_neutron()
    
    # Verify that we have some cross sections
    assert len(macro_xs) > 0, "No macroscopic cross sections were calculated"
    
    # Calculate the total cross section
    total_xs = material.calculate_total_xs_neutron()
    
    # Verify that the total cross section was added
    assert "total" in total_xs, "Total cross section not found in result"
    
    # Verify that the total is the sum of relevant MT reactions
    # Choose a few points to check
    grid = material.unified_energy_grid_neutron()
    
    # Sum up all the relevant MT reactions for a few energy points
    relevant_mts = ["2", "102"]  # Simplified list for testing
    for mt in relevant_mts:
        if mt in macro_xs:
            # Check that values were added to the total (simplified test)
            assert total_xs["total"][0] >= macro_xs[mt][0], f"Total XS should include {mt}"
