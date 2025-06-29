import pytest
from materials_for_mc import Material

def test_unified_energy_grid():
    material = Material()
    material.add_nuclide("Li6", 1.0)
    material.add_nuclide("Li7", 1.0)
    material.read_nuclides_from_json({"Li6": "tests/li6.json", "Li7": "tests/li7.json"})
    # Use known values for your test case
    grid = material.unified_energy_grid("neutron", "294", "2")
    # The grid should be sorted and unique
    assert all(grid[i] < grid[i+1] for i in range(len(grid)-1)), "Grid is not sorted and unique!"
    assert len(grid) > 0, "Grid should not be empty!"
    # Optionally, check that all energies from both nuclides are present
    energies_li6 = material.nuclide_data["Li6"].incident_particle["neutron"]["294"]["2"]["energy"]
    energies_li7 = material.nuclide_data["Li7"].incident_particle["neutron"]["294"]["2"]["energy"]
    for e in energies_li6 + energies_li7:
        assert any(abs(e - g) < 1e-12 for g in grid), f"Energy {e} missing from grid!"
