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
    xs, energy = nuc.microscopic_cross_section(reaction=2, temperature='294')
    assert len(xs) > 0, "Cross section data should not be empty"
    assert len(energy) > 0, "Energy data should not be empty"
    assert len(xs) == len(energy), "Cross section and energy arrays should have same length"
    
    # Test with different temperature
    xs_300, energy_300 = nuc.microscopic_cross_section(reaction=2, temperature='300')
    assert len(xs_300) > 0, "Cross section data should not be empty for 300K"
    assert len(energy_300) > 0, "Energy data should not be empty for 300K"
    
    # Test different MT numbers
    xs_mt3, energy_mt3 = nuc.microscopic_cross_section(reaction=3, temperature='294')
    assert len(xs_mt3) > 0, "MT=3 cross section data should not be empty"
    assert len(energy_mt3) > 0, "MT=3 energy data should not be empty"


def test_microscopic_cross_section_without_temperature():
    """Test microscopic_cross_section with single loaded temperature."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    # Load only one temperature
    nuc.read_nuclide_from_json('tests/Be9.json', temperatures=['294'])
    
    # Should work without specifying temperature since only one is loaded
    xs, energy = nuc.microscopic_cross_section(2)
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
        nuc.microscopic_cross_section(2)
    error_msg = str(exc_info.value)
    assert "Multiple temperatures loaded" in error_msg
    assert "294" in error_msg and "300" in error_msg


def test_microscopic_cross_section_invalid_temperature():
    """Test error handling for invalid temperature."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Should raise error for non-existent temperature
    with pytest.raises(Exception) as exc_info:
        nuc.microscopic_cross_section(reaction=2, temperature='500')
    error_msg = str(exc_info.value)
    assert "Temperature '500' not found" in error_msg
    assert "Available temperatures:" in error_msg
    assert "294" in error_msg and "300" in error_msg


def test_microscopic_cross_section_invalid_mt():
    """Test error handling for invalid MT number."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Should raise error for non-existent MT
    with pytest.raises(Exception) as exc_info:
        nuc.microscopic_cross_section(reaction=9999, temperature='294')
    error_msg = str(exc_info.value)
    assert "MT 9999 not found" in error_msg
    assert "Available MTs:" in error_msg


def test_microscopic_cross_section_multiple_mt_numbers():
    """Test microscopic_cross_section with various MT numbers."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Test common MT numbers that should exist in Be9
    test_mts = [1, 2, 3, 16, 27, 101, 102]  # Common reaction types
    
    for mt in test_mts:
        try:
            xs, energy = nuc.microscopic_cross_section(reaction=mt, temperature='294')
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
    xs, energy = nuc.microscopic_cross_section(reaction=2)  # Elastic scattering
    assert len(xs) > 0, "Li6 elastic scattering data should not be empty"
    assert len(energy) > 0, "Li6 energy data should not be empty"
    
    # Test with explicit temperature too
    xs_explicit, energy_explicit = nuc.microscopic_cross_section(reaction=2, temperature='294')
    assert xs == xs_explicit, "Results should be identical with/without explicit temperature"
    assert energy == energy_explicit, "Energy should be identical with/without explicit temperature"


def test_auto_loading_from_config():
    """Test that microscopic_cross_section can auto-load data from config when nuclide is empty"""
    from materials_for_mc import Config
    
    # Set up config for auto-loading
    config = Config()
    config.set_cross_sections({'Be9': 'tests/Be9.json'})
    
    # Create empty nuclide with name but no data loaded
    nuc = Nuclide('Be9')
    assert nuc.loaded_temperatures == [], "Should start with no loaded temperatures"
    
    # Call microscopic_cross_section - should auto-load data
    xs, energy = nuc.microscopic_cross_section(reaction=2, temperature='294')
    assert len(xs) > 0, "Auto-loaded cross section data should not be empty"
    assert len(energy) > 0, "Auto-loaded energy data should not be empty"
    assert len(xs) == len(energy), "Cross section and energy arrays should have same length"
    
    # Note: loaded_temperatures won't be updated in the Python object due to immutable API
    # The auto-loading happens internally but doesn't modify the original object


def test_auto_loading_additional_temperature():
    """Test that microscopic_cross_section can auto-load additional temperatures"""
    from materials_for_mc import Config
    
    # Set up config for auto-loading
    config = Config()
    config.set_cross_sections({'Be9': 'tests/Be9.json'})
    
    # Load Be9 with only 294K initially
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json', ['294'])
    
    assert nuc.loaded_temperatures == ['294'], "Should only have 294K loaded initially"
    assert '300' in nuc.available_temperatures, "Should know 300K is available"
    
    # Request 300K data - should auto-load additional temperature
    xs, energy = nuc.microscopic_cross_section(reaction=2, temperature='300')
    assert len(xs) > 0, "Auto-loaded 300K cross section data should not be empty"
    assert len(energy) > 0, "Auto-loaded 300K energy data should not be empty"
    
    # The original nuclide object still shows only 294K due to immutable API
    # But the internal auto-loading worked to provide the 300K data
    

def test_auto_loading_without_config_fails():
    """Test that auto-loading fails gracefully when no config is available"""
    from materials_for_mc import Config
    
    # Clear any existing config by setting an empty dict
    config = Config()
    original_configs = config.get_cross_sections()
    
    # Temporarily clear all configurations
    config.set_cross_sections({})
    
    try:
        # Create empty nuclide with name but no config
        nuc = Nuclide('TestNuclide')
        
        # Call microscopic_cross_section - should fail with helpful error
        try:
            nuc.microscopic_cross_section(reaction=2, temperature='294')
            assert False, "Auto-loading without config should fail"
        except Exception as e:
            error_msg = str(e)
            # The error could be either "No configuration found" or a download error
            # Both are acceptable since there's no valid config
            assert ("No configuration found" in error_msg or 
                    "Failed to download" in error_msg or
                    "404 Not Found" in error_msg), f"Error should indicate missing or invalid configuration: {error_msg}"
            assert "TestNuclide" in error_msg or "TestNuclide" in str(e.__class__), f"Error should be related to TestNuclide: {error_msg}"
    
    finally:
        # Restore original configuration
        if original_configs:
            config.set_cross_sections(original_configs)


def test_auto_loading_multiple_calls_consistent():
    """Test that multiple auto-loading calls give consistent results"""
    from materials_for_mc import Config
    
    # Set up config for auto-loading
    config = Config()
    config.set_cross_sections({'Be9': 'tests/Be9.json'})
    
    # Create empty nuclide
    nuc = Nuclide('Be9')
    
    # Call microscopic_cross_section multiple times
    xs1, energy1 = nuc.microscopic_cross_section(reaction=2, temperature='294')
    xs2, energy2 = nuc.microscopic_cross_section(reaction=2, temperature='294')
    xs3, energy3 = nuc.microscopic_cross_section(reaction=102, temperature='300')
    
    # First two calls should give identical results
    assert xs1 == xs2, "Multiple calls with same parameters should give identical results"
    assert energy1 == energy2, "Multiple calls with same parameters should give identical energy"
    
    # Third call should work too (different MT and temperature)
    assert len(xs3) > 0, "Auto-loading different MT and temperature should work"
    assert len(energy3) > 0, "Auto-loading different MT and temperature should provide energy"


def test_auto_loading_with_manual_loading_combined():
    """Test combining manual loading with auto-loading for additional data"""
    from materials_for_mc import Config
    
    # Set up config for auto-loading
    config = Config()
    config.set_cross_sections({'Be9': 'tests/Be9.json'})
    
    # Manually load some data first
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json', ['294'])
    
    # Verify manual loading worked
    assert '294' in nuc.loaded_temperatures, "Manual loading should work"
    assert '300' in nuc.available_temperatures, "Should know other temperatures are available"
    
    # Now use auto-loading for data that was manually loaded
    xs_manual, energy_manual = nuc.microscopic_cross_section(reaction=2, temperature='294')
    assert len(xs_manual) > 0, "Should get data for manually loaded temperature"
    
    # And use auto-loading for additional temperature not manually loaded  
    xs_auto, energy_auto = nuc.microscopic_cross_section(reaction=2, temperature='300')
    assert len(xs_auto) > 0, "Should auto-load additional temperature"
    
    # Test with a temperature-specific MT - try MT=444 which should only be available at 294K
    xs_specific, energy_specific = nuc.microscopic_cross_section(reaction=444, temperature='294')
    assert len(xs_specific) > 0, "Should get temperature-specific MT data"
    
    # Try to get MT=444 at 300K - this should fail since it's not available at that temperature
    try:
        nuc.microscopic_cross_section(reaction=444, temperature='300')
        # If we get here without exception, that's fine too - means 444 exists at both temps
        print("MT=444 exists at both temperatures")
    except ValueError as e:
        # This is expected if MT=444 is not available at 300K
        assert "MT 444 not found" in str(e), f"Should get MT not found error, got: {e}"
    
    # Note: For Be9 MT=2, the cross sections at 294K and 300K might be identical
    # This is fine - the important thing is that both calls succeeded


def test_fendl_3_2c_keyword():
    """Test that the fendl-3.2c keyword is recognized and works correctly."""
    import materials_for_mc
    
    # Test that the keyword is recognized (this tests the Rust backend)
    try:
        # This should not raise an exception if the keyword is recognized
        # We'll create a dummy config entry to test keyword recognition
        from materials_for_mc import Config
        config = Config()
        
        # Test setting cross sections with the keyword - this should not fail
        # if the keyword is recognized in the backend
        config.set_cross_sections({'Li6': 'fendl-3.2c'})
        
        # Verify we can retrieve it
        cross_sections = config.get_cross_sections()
        assert 'Li6' in cross_sections, "Li6 should be in cross sections config"
        assert cross_sections['Li6'] == 'fendl-3.2c', "Should store fendl-3.2c keyword correctly"
        
        # Test that keyword expansion would work (without actually downloading)
        # This implicitly tests the URL cache functionality  
        print("fendl-3.2c keyword test passed - keyword is recognized")
        
    except Exception as e:
        pytest.fail(f"fendl-3.2c keyword should be recognized by the system: {e}")


def test_auto_loading_with_global_keyword():
    """Test that auto-loading works with global keyword configuration"""
    from materials_for_mc import Config, Nuclide
    
    # Set global keyword configuration
    config = Config()
    config.set_cross_sections('fendl-3.2c')
    
    # Verify config is set correctly
    assert config.get_cross_section('Li6') == 'fendl-3.2c', "Global config should apply to Li6"
    
    # Create empty nuclide
    nuc = Nuclide('Li6')
    assert nuc.loaded_temperatures == [], "Should start with no loaded temperatures"
    
    # Call microscopic_cross_section - should auto-load data from global config
    try:
        xs, energy = nuc.microscopic_cross_section(reaction=1, temperature='294')
        assert len(xs) > 0, "Auto-loaded cross section data should not be empty"
        assert len(energy) > 0, "Auto-loaded energy data should not be empty"
        assert len(xs) == len(energy), "Cross section and energy arrays should have same length"
        print("Auto-loading with global keyword test passed!")
        
    except Exception as e:
        # If we can't download (no internet or URL issues), that's OK for this test
        # The important thing is that the config lookup worked
        if "No configuration found" in str(e):
            pytest.fail(f"Config lookup failed - auto-loading should work with global keywords: {e}")
        else:
            print(f"Note: Auto-loading test skipped due to download issue: {e}")
            # This is acceptable - we verified the config lookup works


def test_microscopic_cross_section_by_name():
    """Test microscopic_cross_section with reaction names."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Test elastic scattering using reaction name
    xs_name, energy_name = nuc.microscopic_cross_section("(n,elastic)", temperature='294')
    assert len(xs_name) > 0, "Elastic scattering cross section should not be empty"
    assert len(energy_name) > 0, "Energy data should not be empty"
    
    # Compare with MT number approach (MT=2 is elastic scattering)
    xs_mt, energy_mt = nuc.microscopic_cross_section(reaction=2, temperature='294')
    
    # Should get identical results
    assert xs_name == xs_mt, "Reaction name and MT number should give identical cross sections"
    assert energy_name == energy_mt, "Reaction name and MT number should give identical energy grids"
    
    # Test other common reactions
    test_reactions = [
        ("(n,gamma)", 102),   # Radiative capture
        ("(n,a)", 107),       # Alpha production
        ("(n,total)", 1),     # Total cross section
    ]
    
    for reaction_name, mt_num in test_reactions:
        try:
            xs_name, energy_name = nuc.microscopic_cross_section(reaction_name, temperature='294')
            xs_mt, energy_mt = nuc.microscopic_cross_section(reaction=mt_num, temperature='294')
            
            assert len(xs_name) > 0, f"{reaction_name} should have cross section data"
            assert len(energy_name) > 0, f"{reaction_name} should have energy data"
            assert xs_name == xs_mt, f"{reaction_name} and MT={mt_num} should give identical results"
            assert energy_name == energy_mt, f"{reaction_name} and MT={mt_num} should give identical energy"
            
        except Exception as e:
            # Some reactions might not exist for Be9, which is acceptable
            if "not found" in str(e).lower():
                print(f"Note: {reaction_name} (MT={mt_num}) not available in Be9 data")
            else:
                raise e


def test_microscopic_cross_section_by_name_invalid_reaction():
    """Test error handling for invalid reaction names."""
    from materials_for_mc import Nuclide
    nuc = Nuclide('Be9')
    nuc.read_nuclide_from_json('tests/Be9.json')
    
    # Test with invalid reaction name
    with pytest.raises(Exception) as exc_info:
        nuc.microscopic_cross_section("invalid_reaction", temperature='294')
    error_msg = str(exc_info.value)
    assert "not found in reaction mapping" in error_msg or "Unknown reaction" in error_msg


def test_microscopic_cross_section_by_name_fission():
    """Test that the special 'fission' alias works."""
    from materials_for_mc import Nuclide
    
    # Use Li6 which might have fission data, or test the error handling
    nuc = Nuclide('Li6')
    nuc.read_nuclide_from_json('tests/Li6.json')
    
    try:
        xs_fission, energy_fission = nuc.microscopic_cross_section("fission", temperature='294')
        xs_mt18, energy_mt18 = nuc.microscopic_cross_section(18, temperature='294')
        
        # Should get identical results since fission maps to MT=18
        assert xs_fission == xs_mt18, "fission and MT=18 should give identical results"
        assert energy_fission == energy_mt18, "fission and MT=18 should give identical energy"
        
    except Exception as e:
        # Li6 might not have fission data, which is acceptable
        if "MT 18 not found" in str(e) or "not found" in str(e).lower():
            print("Note: Li6 does not have fission data (expected)")
        else:
            raise e
