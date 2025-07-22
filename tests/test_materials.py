import pytest
import gc
import sys
from materials_for_mc import Material, Materials, Config

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
    from materials_for_mc import Config
    
    # Set the cross-section paths in the global Config
    Config.set_cross_sections({"Li6": "tests/Li6.json", "Li7": "tests/Li7.json"})
    
    material1 = Material()
    material1.add_nuclide("Li6", 1.0)

    material2 = Material()
    material2.add_nuclide("Li7", 1.0)
    
    # Create a Materials collection
    materials = Materials([material1, material2])
    
    # Accessing data will automatically load nuclides from the global Config
    assert len(materials) == 2

