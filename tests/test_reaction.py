def test_reaction_instantiation():
    from materials_for_mc import Reaction
    r = Reaction(["n"], ["n", "n"], 1.23)
    assert r.reactants == ["n"]
    assert r.products == ["n", "n"]
    assert r.energy == 1.23
