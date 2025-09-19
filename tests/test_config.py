import pytest
import materials_for_mc as m4mc

@pytest.fixture(autouse=True)
def clear_config():
    """Clear config state before each test"""
    m4mc.Config.clear()
    yield
    # No cleanup needed after test

def test_set_cross_sections_with_dict():
    """Test setting cross sections with a dictionary"""
    cross_sections = {
        "Li6": "tendl-21",
        "Li7": "../../tests/Li7.json"
    }
    m4mc.Config.set_cross_sections(cross_sections)
    
    # Verify the cross sections were set
    assert m4mc.Config.get_cross_section("Li6") == "tendl-21"
    assert m4mc.Config.get_cross_section("Li7") == "../../tests/Li7.json"

def test_set_cross_sections_with_string():
    """Test setting cross sections with a global keyword string"""
    m4mc.Config.set_cross_sections("tendl-21")
    
    # Any nuclide should now return the global keyword
    assert m4mc.Config.get_cross_section("Fe56") == "tendl-21"
    assert m4mc.Config.get_cross_section("Li6") == "tendl-21"

def test_set_cross_section_single_nuclide_path():
    """Test setting a single nuclide with a file path"""
    m4mc.Config.set_cross_section("Fe56", "../../tests/Fe56.json")
    assert m4mc.Config.get_cross_section("Fe56") == "../../tests/Fe56.json"

def test_set_cross_section_single_nuclide_keyword():
    """Test setting a single nuclide with a keyword"""
    m4mc.Config.set_cross_section("Fe56", "tendl-21")
    assert m4mc.Config.get_cross_section("Fe56") == "tendl-21"

def test_set_cross_section_global_keyword():
    """Test setting a global keyword using set_cross_section"""
    m4mc.Config.set_cross_section("tendl-21")
    assert m4mc.Config.get_cross_section("Li6") == "tendl-21"
    assert m4mc.Config.get_cross_section("Fe56") == "tendl-21"

def test_set_cross_sections_invalid_type():
    """Test that set_cross_sections raises TypeError for invalid input"""
    with pytest.raises(TypeError):
        m4mc.Config.set_cross_sections(123)  # Invalid type

def test_mixed_global_and_specific_config():
    """Test mixing global default with specific nuclide overrides"""
    # Set global default to TENDL
    m4mc.Config.set_cross_sections("tendl-21")
    
    # Override specific nuclides to FENDL
    m4mc.Config.set_cross_section("Fe56", "fendl-3.2c")
    m4mc.Config.set_cross_section("Li6", "tests/Li6.json")
    
    # Check that specific overrides work
    assert m4mc.Config.get_cross_section("Fe56") == "fendl-3.2c"
    assert m4mc.Config.get_cross_section("Li6") == "tests/Li6.json"
    
    # Check that other nuclides fall back to global default
    assert m4mc.Config.get_cross_section("Be9") == "tendl-21"
    assert m4mc.Config.get_cross_section("U235") == "tendl-21"

def test_explicit_path_override_in_nuclide_loading():
    """Test that explicit paths in read_nuclide_from_json override global config"""
    # Set up global config
    m4mc.Config.set_cross_sections({
        "Li6": "tendl-21",
        "Be9": "fendl-3.2c"
    })
    
    # Create nuclides that will override the config with explicit paths
    li6_from_config = m4mc.Nuclide("Li6")
    li6_from_config.read_nuclide_from_json()  # Should use tendl-21 from config
    
    li6_explicit = m4mc.Nuclide("Li6") 
    li6_explicit.read_nuclide_from_json("tests/Li6.json")  # Should use explicit path
    
    # Both should load successfully but potentially from different sources
    assert li6_from_config.name == "Li6"
    assert li6_explicit.name == "Li6"
    
    # Verify they loaded (both should have data)
    assert len(li6_from_config.available_temperatures) > 0
    assert len(li6_explicit.available_temperatures) > 0

# def test_set_cross_section_invalid_keyword():
#     """Test that set_cross_section raises error for invalid keyword"""
#     import pytest
#     # This should raise a panic that gets converted to a Python exception
#     with pytest.raises(Exception) as exc_info:
#         m4mc.Config.set_cross_section("invalid-keyword")
#     assert "Invalid cross section keyword" in str(exc_info.value)