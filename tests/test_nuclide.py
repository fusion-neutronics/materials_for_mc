import pytest
from materials_for_mc import Nuclide

def test_be9_not_fissionable():
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    assert hasattr(nuc, 'fissionable'), "Nuclide should have a 'fissionable' attribute"
    assert nuc.fissionable is False, "Be9 should not be fissionable"

def test_fe58_not_fissionable():
    nuc = Nuclide('Fe58')
    nuc.read_nuclide_from_json('tests/Fe58.json')
    assert hasattr(nuc, 'fissionable'), "Nuclide should have a 'fissionable' attribute"
    assert nuc.fissionable is False, "Fe58 should not be fissionable"
from materials_for_mc import Nuclide

def test_read_li6_nuclide():
    nuc1 = Nuclide('Li6')
    nuc1.read_nuclide_from_json('tests/Li6.json')
    assert nuc1.element.lower() == 'lithium'
    assert nuc1.atomic_symbol == "Li"
    assert nuc1.atomic_number == 3
    assert nuc1.mass_number == 6
    assert nuc1.neutron_number == 3
    assert nuc1.available_temperatures == ['294']
    # We don't expect any specific order of MT numbers, just check they're all ints
    assert all(isinstance(mt, int) for mt in nuc1.reaction_mts)

    cs = nuc1.reactions['294'][2]['cross_section']

    for entry in cs:
        assert isinstance(entry, float)
        assert isinstance(entry, float)

    expected_li6 = [1, 2, 3, 4, 5, 24, 27, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 101, 102, 103, 105, 203, 204, 205, 206, 207, 301, 444]
    assert nuc1.reaction_mts == expected_li6

def test_read_li7_nuclide():
    nuc1 = Nuclide('Li7')
    nuc1.read_nuclide_from_json('tests/Li7.json')
    assert nuc1.element.lower() == 'lithium'
    assert nuc1.atomic_symbol == "Li"
    assert nuc1.atomic_number == 3
    assert nuc1.mass_number == 7
    assert nuc1.neutron_number == 4
    assert nuc1.available_temperatures == ['294']
    # We don't expect any specific order of MT numbers, just check they're all ints
    assert all(isinstance(mt, int) for mt in nuc1.reaction_mts)

    cs = nuc1.reactions['294'][2]['cross_section']
    
    for entry in cs:
        assert isinstance(entry, float)
        assert isinstance(entry, float)

    expected_li7 = [1, 2, 3, 4, 5, 16, 24, 25, 27, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 101, 102, 104, 203, 204, 205, 206, 207, 301, 444]
    assert nuc1.reaction_mts == expected_li7


def test_read_be9_available_and_loaded_temperatures():
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    assert nuc.available_temperatures == ['294', '300']
    # By current implementation, all temps are loaded eagerly
    assert hasattr(nuc, 'loaded_temperatures'), "loaded_temperatures attribute missing"
    assert nuc.loaded_temperatures == ['294', '300']
    # Reactions dict should contain both temperatures
    assert '294' in nuc.reactions
    assert '300' in nuc.reactions


def test_read_be9_mt_numbers_per_temperature():
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    mts_294 = sorted(int(mt) for mt in nuc.reactions['294'].keys())
    mts_300 = sorted(int(mt) for mt in nuc.reactions['300'].keys())
    expected_294 = sorted([
        1,2,3,16,27,101,102,103,104,105,107,203,204,205,207,301,444,
        875,876,877,878,879,880,881,882,883,884,885,886,887,888,889,890
    ])
    expected_300 = sorted([
        1,2,3,16,27,101,102,103,104,105,107,203,204,205,207,301
    ])
    assert mts_294 == expected_294, f"Be9 294K MT list mismatch: {mts_294}"
    assert mts_300 == expected_300, f"Be9 300K MT list mismatch: {mts_300}"
    # Ensure 300K list is subset of 294K list
    assert set(mts_300).issubset(set(mts_294))


def test_read_be9_selective_single_temperature():
    # Ensure only the specified temperature (300) is retained in reactions and loaded_temperatures
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json', temperatures=['300'])
    assert nuc.available_temperatures == ['294', '300'], "available_temperatures should list all temps present in file"
    assert nuc.loaded_temperatures == ['300'], f"loaded_temperatures should be only ['300'], got {nuc.loaded_temperatures}"
    assert '300' in nuc.reactions, "300K reactions missing after selective load"
    assert '294' not in nuc.reactions, "294K reactions should not be loaded when selectively requesting only 300K"
    # MT list at 300 should match subset expectation
    mts_300 = sorted(int(mt) for mt in nuc.reactions['300'].keys())
    expected_300 = sorted([
        1,2,3,16,27,101,102,103,104,105,107,203,204,205,207,301
    ])
    assert mts_300 == expected_300, f"Selective load 300K MT list mismatch: {mts_300}"


def test_read_nuclide_from_json_keyword():
    from materials_for_mc import Nuclide
    nuc = Nuclide('Li6')
    nuc.read_nuclide_from_json("tendl-21")

def test_read_nuclide_from_json_local_path():
    from materials_for_mc import Nuclide
    nuc = Nuclide('Li6')
    # Should not raise TypeError when passing local path
    nuc.read_nuclide_from_json("tests/Li6.json")
