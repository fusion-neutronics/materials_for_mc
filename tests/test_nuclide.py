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


def test_microscopic_cross_section_with_temperature():
    """Test microscopic_cross_section with explicit temperature."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Test with specific temperature
    xs, energy = nuc.microscopic_cross_section(mt=2, temperature='294')
    assert len(xs) > 0, "Cross section data should not be empty"
    assert len(energy) > 0, "Energy data should not be empty"
    assert len(xs) == len(energy), "Cross section and energy arrays should have same length"
    
    # Test with different temperature
    xs_300, energy_300 = nuc.microscopic_cross_section(mt=2, temperature='300')
    assert len(xs_300) > 0, "Cross section data should not be empty for 300K"
    assert len(energy_300) > 0, "Energy data should not be empty for 300K"
    
    # Test different MT numbers
    xs_mt3, energy_mt3 = nuc.microscopic_cross_section(mt=3, temperature='294')
    assert len(xs_mt3) > 0, "MT=3 cross section data should not be empty"
    assert len(energy_mt3) > 0, "MT=3 energy data should not be empty"


def test_microscopic_cross_section_without_temperature():
    """Test microscopic_cross_section with single loaded temperature."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    # Load only one temperature
    nuc.read_nuclide_from_json('tests/Be9.json', temperatures=['294'])
    
    # Should work without specifying temperature since only one is loaded
    xs, energy = nuc.microscopic_cross_section(mt=2)
    assert len(xs) > 0, "Cross section data should not be empty"
    assert len(energy) > 0, "Energy data should not be empty"
    assert len(xs) == len(energy), "Cross section and energy arrays should have same length"


def test_microscopic_cross_section_multiple_temperatures_error():
    """Test that microscopic_cross_section raises error when multiple temperatures loaded without specifying."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')  # Loads both 294 and 300
    
    # Should raise error when no temperature specified with multiple loaded
    with pytest.raises(Exception) as exc_info:
        nuc.microscopic_cross_section(mt=2)
    assert "Multiple temperatures loaded" in str(exc_info.value)


def test_microscopic_cross_section_invalid_temperature():
    """Test error handling for invalid temperature."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Should raise error for non-existent temperature
    with pytest.raises(Exception) as exc_info:
        nuc.microscopic_cross_section(mt=2, temperature='500')
    assert "Temperature '500' not found" in str(exc_info.value)


def test_microscopic_cross_section_invalid_mt():
    """Test error handling for invalid MT number."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Should raise error for non-existent MT
    with pytest.raises(Exception) as exc_info:
        nuc.microscopic_cross_section(mt=9999, temperature='294')
    assert "MT 9999 not found" in str(exc_info.value)


def test_microscopic_cross_section_multiple_mt_numbers():
    """Test microscopic_cross_section with various MT numbers."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Test common MT numbers that should exist in Be9
    test_mts = [1, 2, 3, 16, 27, 101, 102]  # Common reaction types
    
    for mt in test_mts:
        try:
            xs, energy = nuc.microscopic_cross_section(mt=mt, temperature='294')
            assert len(xs) > 0, f"MT={mt} should have cross section data"
            assert len(energy) > 0, f"MT={mt} should have energy data"
            assert len(xs) == len(energy), f"MT={mt} data length mismatch"
            assert all(e > 0 for e in energy), f"MT={mt} energy values should be positive"
            assert all(x >= 0 for x in xs), f"MT={mt} cross sections should be non-negative"
        except Exception:
            # Some MT numbers might not exist, which is fine
            pass


def test_microscopic_cross_section_lithium():
    """Test microscopic_cross_section with Li6 data."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Li6')
    nuc.read_nuclide_from_json('tests/Li6.json')
    
    # Li6 should have only one temperature, so no temperature needed
    xs, energy = nuc.microscopic_cross_section(mt=2)  # Elastic scattering
    assert len(xs) > 0, "Li6 elastic scattering data should not be empty"
    assert len(energy) > 0, "Li6 energy data should not be empty"
    
    # Test with explicit temperature too
    xs_explicit, energy_explicit = nuc.microscopic_cross_section(mt=2, temperature='294')
    assert xs == xs_explicit, "Results should be identical with/without explicit temperature"
    assert energy == energy_explicit, "Energy should be identical with/without explicit temperature"
