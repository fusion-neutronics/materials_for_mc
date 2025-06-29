from materials_for_mc import Nuclide

def test_read_li6_nuclide():
    nuc1 = Nuclide('Li6')
    nuc1.read_nuclide_from_json('tests/li6.json')
    # assert nuc1.data is not None
    # data = nuc1.data
    # assert data.element.lower() == 'lithium'
    # assert data.atomic_symbol == "Li"
    # assert data.proton_number == 3
    # assert data.mass_number == 6
    # assert data.neutron_number == 3
    # assert data.incident_particle == "neutron"
    # assert isinstance(data.temperature, list)
    # assert len(data.temperature) > 0
    # # Check at least one temperature entry and one reaction
    # temp_entry = data.temperature[0]
    # assert isinstance(temp_entry.temps, dict)
    # for t, reactions in temp_entry.temps.items():
    #     assert isinstance(reactions, list)
    #     if reactions:
    #         reaction = reactions[0]
    #         assert hasattr(reaction, "reaction_products")
    #         assert hasattr(reaction, "mt_reaction_number")
    #         assert isinstance(reaction.cross_section, list)
    #         assert isinstance(reaction.energy, list)
    #         break