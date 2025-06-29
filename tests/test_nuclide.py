from materials_for_mc import Nuclide

def test_read_li6_nuclide():
    nuc1 = Nuclide('Li6')
    nuc1.read_nuclide_from_json('tests/li6.json')
    assert nuc1.element.lower() == 'lithium'
    assert nuc1.atomic_symbol == "Li"
    assert nuc1.proton_number == 3
    assert nuc1.mass_number == 6
    assert nuc1.neutron_number == 3
    assert list(nuc1.incident_particle.keys())[0] == "neutron"
    assert nuc1.incident_particles == ['neutron']
    assert nuc1.temperatures == ['294']
    assert nuc1.reaction_mts[0] == 1

    cs = nuc1.incident_particle['neutron']['294']['2']['cross_section']
    energy = nuc1.incident_particle['neutron']['294']['2']['energy']

    for entry in cs:
        assert isinstance(entry, float)
    for entry in energy:
        assert isinstance(entry, float)
