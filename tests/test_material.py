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