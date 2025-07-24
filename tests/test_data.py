import pytest
from materials_for_mc import natural_abundance, element_nuclides

def test_lithium_natural_abundance():
    abund = natural_abundance()
    li6 = float(abund['Li6'])
    li7 = float(abund['Li7'])
    assert abs(li6 - 0.0759) < 1e-4, f"Li6 abundance incorrect: {li6}"
    assert abs(li7 - 0.9241) < 1e-4, f"Li7 abundance incorrect: {li7}"
    assert abs(li6 + li7 - 1.0) < 1e-3, f"Li6 + Li7 should sum to 1, got {li6 + li7}"

def test_element_nuclides_li_and_be():
    nuclides = element_nuclides()
    assert sorted(nuclides['Li']) == ['Li6', 'Li7']
    assert sorted(nuclides['Be']) == ['Be9']
