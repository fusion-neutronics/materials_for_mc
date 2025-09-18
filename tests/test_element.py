#!/usr/bin/env python3
"""
Tests for the element module Python bindings.
"""
import pytest
import os
import sys

# Add the package to the Python path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

import materials_for_mc as m4mc

def test_element_class():
    """Test the Element class functionality."""
    # Create elements with different symbols
    h = m4mc.Element("H")
    fe = m4mc.Element("Fe")
    u = m4mc.Element("U")
    
    # Check that the element names are correctly stored
    assert h.name == "H"
    assert fe.name == "Fe"
    assert u.name == "U"
    
    # Test getting nuclides for each element
    h_nuclides = h.get_nuclides()
    fe_nuclides = fe.get_nuclides()
    u_nuclides = u.get_nuclides()
    
    # Hydrogen should have at least H1 and H2 (protium and deuterium)
    assert len(h_nuclides) >= 2
    assert "H1" in h_nuclides
    assert "H2" in h_nuclides
    
    # Iron should have its natural isotopes
    assert len(fe_nuclides) >= 4  # Fe54, Fe56, Fe57, Fe58
    assert "Fe54" in fe_nuclides
    assert "Fe56" in fe_nuclides
    assert "Fe57" in fe_nuclides
    assert "Fe58" in fe_nuclides
    
    # Uranium should have at least U235 and U238
    assert len(u_nuclides) >= 2
    assert "U235" in u_nuclides
    assert "U238" in u_nuclides

def test_get_nuclides():
    """Test per-element get_nuclides method returns list for that element."""
    al_isotopes = m4mc.Element('Al').get_nuclides()
    assert isinstance(al_isotopes, list)
    assert all(iso.startswith('Al') for iso in al_isotopes)
    assert len(al_isotopes) > 0

def test_nonexistent_element():
    """Test behavior with a non-existent element."""
    # Create an element with a symbol that doesn't exist
    fake_element = m4mc.Element("Zz")
    
    # Should return an empty list of nuclides
    assert fake_element.get_nuclides() == []

def test_atomic_number():
    """Test that atomic numbers are correctly assigned to elements."""
    # Test common elements
    h = m4mc.Element("H")
    assert h.atomic_number == 1
    
    li = m4mc.Element("Li")
    assert li.atomic_number == 3
    
    fe = m4mc.Element("Fe")
    assert fe.atomic_number == 26
    
    u = m4mc.Element("U")
    assert u.atomic_number == 92
    
    # Test unknown element has no atomic number
    fake = m4mc.Element("Zz")
    assert fake.atomic_number is None

def test_microscopic_cross_section():
    """Test microscopic cross section calculation for elements."""
    # Configure data sources for testing
    m4mc.Config.set_cross_sections({
        "Li6": "tests/Li6.json",
        "Li7": "tests/Li7.json",
    })
    
    # Test with lithium element
    li = m4mc.Element("Li")
    
    # Test (n,gamma) reaction
    energy, xs = li.microscopic_cross_section("(n,gamma)")
    
    # Verify we get valid data
    assert isinstance(energy, list)
    assert isinstance(xs, list)
    assert len(energy) == len(xs)
    assert len(energy) > 0
    
    # Verify energy values are positive and sorted
    assert all(e > 0 for e in energy)
    assert energy == sorted(energy)
    
    # Verify cross section values are non-negative
    assert all(x >= 0 for x in xs)
    
    # Test MT number format
    energy_mt, xs_mt = li.microscopic_cross_section(102)  # 102 is (n,gamma)
    
    # Results should be identical for string and MT number
    assert energy == energy_mt
    assert xs == xs_mt

def test_microscopic_cross_section_with_temperature():
    """Test microscopic cross section calculation with temperature specification."""
    # Configure data sources
    m4mc.Config.set_cross_sections({
        "Li6": "tests/Li6.json",
        "Li7": "tests/Li7.json",
    })
    
    li = m4mc.Element("Li")
    
    # Test with temperature parameter
    energy, xs = li.microscopic_cross_section("(n,gamma)", temperature="294")
    
    assert len(energy) > 0
    assert len(xs) == len(energy)
    assert all(e > 0 for e in energy)
    assert all(x >= 0 for x in xs)

def test_microscopic_cross_section_error_conditions():
    """Test error handling for microscopic cross section calculations."""
    # Test with element that has no isotopes
    fake_element = m4mc.Element("Zz")
    
    with pytest.raises(Exception) as exc_info:
        fake_element.microscopic_cross_section("(n,gamma)")
    
    assert "No known isotopes" in str(exc_info.value)
    
    # Test with invalid reaction name
    li = m4mc.Element("Li")
    with pytest.raises(Exception) as exc_info:
        li.microscopic_cross_section("invalid_reaction_name")
    
    # Should mention something about the reaction not being found
    assert any(keyword in str(exc_info.value).lower() for keyword in 
               ["reaction", "not found", "invalid", "error"])

if __name__ == "__main__":
    pytest.main(["-v", __file__])
