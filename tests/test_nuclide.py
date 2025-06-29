import pytest
import os
import json
from materials_for_mc import nuclide_python

# Example test for loading a nuclide from JSON

def test_read_li6_nuclide():
    # Path to the test JSON file
    json_path = os.path.join(os.path.dirname(__file__), 'li6.json')
    nuclide = nuclide_python.py_read_nuclide_from_json(json_path)
    assert nuclide.element.lower() == 'li' or nuclide.atomic_symbol.lower() == 'li'
    assert nuclide.mass_number == 6
    assert isinstance(nuclide.temperature, float)
    assert isinstance(nuclide.reactions, dict)
    # Check at least one reaction exists
    assert len(nuclide.reactions) > 0
    for mt, reaction in nuclide.reactions.items():
        assert isinstance(mt, int)
        assert isinstance(reaction.energies, list)
        assert isinstance(reaction.cross_sections, list)
        assert all(isinstance(e, float) for e in reaction.energies)
        assert all(isinstance(cs, float) for cs in reaction.cross_sections)


def test_read_li7_nuclide():
    json_path = os.path.join(os.path.dirname(__file__), 'li7.json')
    nuclide = nuclide_python.py_read_nuclide_from_json(json_path)
    assert nuclide.mass_number == 7
    assert isinstance(nuclide.temperature, float)
    assert isinstance(nuclide.reactions, dict)
    assert len(nuclide.reactions) > 0
