import pytest
from materials_for_mc import Material, Materials

def test_building_up_materials():
    mat1 = Material()
    mat1.add_nuclide("H", 1.0)
    
    mat2 = Material()
    mat2.add_nuclide("Fe", 0.5)
    
    # mats = Materials([mat1, mat2])
    mats= Materials()
    mats.append(mat1)
    mats.append(mat2)
    assert len(mats) == 2
    assert mats[0].nuclides == [("H", 1.0)]
    assert mats[1].nuclides == [("Fe", 0.5)]


def test_building_up_materials():
    mat1 = Material()
    mat1.add_nuclide("H", 1.0)
    
    mat2 = Material()
    mat2.add_nuclide("Fe", 0.5)
    
    # mats = Materials([mat1, mat2])
    mats= Materials([mat1, mat2])
    assert len(mats) == 2
    assert mats[0].nuclides == [("H", 1.0)]
    assert mats[1].nuclides == [("Fe", 0.5)]

def test_material_data_reading():
    material = Material()
    material.add_nuclide("Li6", 1.0)
    
    # Assuming the JSON file is structured correctly
    material.read_nuclides_from_json({"Li6": "tests/li6.json"})
    
    assert len(material.nuclides) == 1
    assert material.nuclides[0].element.lower() == "li"
    assert material.nuclides[0].mass_number == 6
    assert isinstance(material.nuclides[0].temperature, float)
    assert isinstance(material.nuclides[0].reactions, dict)
    assert len(material.nuclides[0].reactions) > 0