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
    # We don't expect any specific order of MT numbers, just check they're all strings
    assert all(isinstance(mt, str) for mt in nuc1.reaction_mts)

    cs = nuc1.reactions['294']['2']['cross_section']
    
    for entry in cs:
        assert isinstance(entry, float)
        assert isinstance(entry, float)
