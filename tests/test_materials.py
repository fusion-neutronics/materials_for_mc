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

def test_materials_data_reading():
    material1 = Material()
    material1.add_nuclide("Li6", 1.0)

    material2 = Material()
    material2.add_nuclide("Li7", 1.0)
    
    # Assuming the JSON file is structured correctly
    materials = Materials([material1, material2])
    materials.read_nuclides_from_json({"Li6": "tests/li6.json"})
