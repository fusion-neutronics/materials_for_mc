import pytest
from materials_for_mc import Material

def test_unified_energy_grid_neutron():
    material = Material()
    material.add_nuclide("Li6", 1.0)
    material.add_nuclide("Li7", 1.0)
    material.temperature = "294"  # Set temperature using the property
    material.read_nuclides_from_json({"Li6": "tests/li6_neutron.json", "Li7": "tests/li7_neutron.json"})
    # Get the unified energy grid across all MT reactions
    grid = material.unified_energy_grid_neutron()
    # The grid should be sorted and unique
    assert all(grid[i] < grid[i+1] for i in range(len(grid)-1)), "Grid is not sorted and unique!"
    assert len(grid) > 0, "Grid should not be empty!"
    # Optionally, check that specific energies are present (if you know some expected values)
    # This is just a basic check that the grid contains data
    assert len(grid) > 100, "Grid should contain a significant number of energy points!"
