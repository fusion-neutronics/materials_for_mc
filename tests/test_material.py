import pytest
from materials_for_mc import Material

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