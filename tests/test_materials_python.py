import pytest
import materials_for_mc as m4mc

def test_read_nuclides_from_json_keyword():
    mat = m4mc.Material()
    mat.add_element('Li', 1.0)
    mat.set_density('g/cm3', 2.0)
    mat.volume = 1.0
    # Should not raise TypeError
    try:
        mat.read_nuclides_from_json("tendl-21")
    except TypeError:
        pytest.fail("TypeError raised when passing keyword string to read_nuclides_from_json")

def test_read_nuclides_from_json_dict():
    mat = m4mc.Material()
    mat.add_element('Li', 1.0)
    mat.set_density('g/cm3', 2.0)
    mat.volume = 1.0
    # Should not raise TypeError
    try:
        mat.read_nuclides_from_json({"Li6": "tendl-21"})
    except TypeError:
        pytest.fail("TypeError raised when passing dict to read_nuclides_from_json")
