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
    assert nuc1.temperatures == ['294']
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
    assert nuc1.temperatures == ['294']
    # We don't expect any specific order of MT numbers, just check they're all ints
    assert all(isinstance(mt, int) for mt in nuc1.reaction_mts)

    cs = nuc1.reactions['294'][2]['cross_section']
    
    for entry in cs:
        assert isinstance(entry, float)
        assert isinstance(entry, float)

    expected_li7 = [1, 2, 3, 4, 5, 16, 24, 25, 27, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 101, 102, 104, 203, 204, 205, 206, 207, 301, 444]
    assert nuc1.reaction_mts == expected_li7