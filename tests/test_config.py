import pytest
import materials_for_mc as m4mc

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

# def test_set_cross_section_invalid_keyword():
#     """Test that set_cross_section raises error for invalid keyword"""
#     import pytest
#     # This should raise a panic that gets converted to a Python exception
#     with pytest.raises(Exception) as exc_info:
#         m4mc.Config.set_cross_section("invalid-keyword")
#     assert "Invalid cross section keyword" in str(exc_info.value)