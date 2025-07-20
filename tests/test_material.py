import pytest
from materials_for_mc import Material, Config

def test_volume_set_and_get():
    material = Material()

    # Test setting a valid volume
    material.volume = 100.0
    assert material.volume == 100.0

    # Test setting another valid volume
    material.volume = 200.0
    assert material.volume == 200.0

    # Test setting an invalid (negative) volume
    with pytest.raises(ValueError, match="Volume must be positive"):
        material.volume = -50.0

    # Ensure the volume remains unchanged after the invalid set
    assert material.volume == 200.0

def test_initial_volume():
    material = Material()

    # Test that the initial volume is None
    assert material.volume is None
    
def test_adding_nuclides():
    material = Material()
    material.add_nuclide("H", 1.0)
    material.add_nuclide("Fe", 0.5)
    assert material.nuclides == [("Fe", 0.5), ("H", 1.0)] # TODO change to named tuple using a custom type

def test_density_settings():
    material = Material()
    material.add_nuclide("W", 0.5)
    material.set_density('g/cm3', 19.3)
    assert material.density == 19.3
    assert material.density_units == 'g/cm3'

def test_get_nuclide_names():
    material = Material()
    
    # Test empty material has no nuclides
    assert material.get_nuclide_names() == []
    
    # Add some nuclides
    material.add_nuclide("U235", 0.05)
    material.add_nuclide("U238", 0.95)
    material.add_nuclide("O16", 2.0)
    
    # Check we get all nuclides in alphabetical order
    assert material.get_nuclide_names() == ["O16", "U235", "U238"]
    
    # Test adding the same nuclide again replaces the previous value
    material.add_nuclide("U235", 0.1)
    assert "U235" in material.get_nuclide_names()
    assert len(material.get_nuclide_names()) == 3  # Still 3 nuclides
    
    # Verify the order is still alphabetical
    assert material.get_nuclide_names() == ["O16", "U235", "U238"]

def test_material_data_xs_reading():
    # Import Config locally to avoid module-level import issues
    from materials_for_mc import Config
    import os
    
    material = Material()
    material.add_nuclide("Li6", 1.0)
    
    # Print the current working directory and check if the file exists
    print(f"Current working directory: {os.getcwd()}")
    print(f"File exists: {os.path.exists('tests/li6_neutron.json')}")
    
    # Set the cross-section path in the global Config
    Config.set_cross_sections({"Li6": "tests/li6_neutron.json"})
    
    # Print the cross-sections to verify they're set correctly
    print(f"Cross-sections: {Config.get_cross_sections()}")
    
    # Accessing methods that need nuclide data will automatically load them
    # Let's call a method that triggers the loading
    grid = material.unified_energy_grid_neutron()
    
    assert len(material.nuclides) == 1
    assert len(grid) > 0, "Energy grid should not be empty"

def test_add_element_lithium():
    mat = Material()
    mat.add_element('Li', 1.0)
    nuclides = dict(mat.nuclides)
    # Should contain Li6 and Li7 with correct fractions
    assert 'Li6' in nuclides
    assert 'Li7' in nuclides
    # Natural abundances from IUPAC
    assert abs(nuclides['Li6'] - 0.07589) < 1e-5
    assert abs(nuclides['Li7'] - 0.92411) < 1e-5


def test_add_element_not_found():
    mat = Material()
    with pytest.raises(Exception) as excinfo:
        mat.add_element('Xx', 1.0)
    assert 'not found' in str(excinfo.value)
