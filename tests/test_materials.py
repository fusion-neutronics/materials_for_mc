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


def test_materials_be9_selective_temperature_union_no_extra():
    from materials_for_mc import Config, clear_nuclide_cache
    # Ensure clean cache so previous tests that may have loaded multiple temps don't leak
    clear_nuclide_cache()
    # Two materials at 294 K; first uses Be9 (has 294 & 300), second uses Fe56
    m1 = Material()
    m1.temperature = "294"
    m1.add_nuclide("Be9", 1.0)
    m2 = Material()
    m2.temperature = "294"
    m2.add_nuclide("Fe56", 1.0)
    Config.set_cross_sections({"Be9": "tests/Be9.json", "Fe56": "tests/Fe56.json"})
    mats = Materials([m1, m2])  # eager union should only request 294 for Be9
    # Explicitly trigger eager union load via read_nuclides_from_json
    mats.read_nuclides_from_json({"Be9": "tests/Be9.json", "Fe56": "tests/Fe56.json"})
    # Inspect loaded temperatures via one of the material wrappers
    temps = mats[0].nuclide_loaded_temperatures("Be9")
    assert temps == ["294"], f"Expected only ['294'] loaded for Be9 across materials, got {temps}"


def test_materials_be9_selective_temperature_union():
    from materials_for_mc import Config, clear_nuclide_cache
    # Ensure clean cache so previous tests that may have loaded multiple temps don't leak
    clear_nuclide_cache()
    # Two materials, both at 294 K, each referencing Be9
    m1 = Material()
    m1.temperature = "294"
    m1.add_nuclide("Be9", 1.0)
    m2 = Material()
    m2.temperature = "300"
    m2.add_nuclide("Be9", 2.0)
    Config.set_cross_sections({"Be9": "tests/Be9.json"})
    mats = Materials([m1, m2])
    # Explicitly trigger eager union load across temperatures 294 & 300
    mats.read_nuclides_from_json({"Be9": "tests/Be9.json"})
    # Inspect loaded temperatures via one of the material wrappers
    temps = mats[0].nuclide_loaded_temperatures("Be9")
    assert temps == ["294", "300"], f"Expected only ['294', '300'] loaded for Be9 across materials, got {temps}"