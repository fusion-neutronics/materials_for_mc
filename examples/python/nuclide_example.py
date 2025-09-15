import materials_for_mc as m4mc
m4mc.Config.set_cross_sections({
    "Be9": "tests/Be9.json",
    "Fe54": "tests/Fe54.json",
    "Fe56": "tests/Fe56.json",
    "Fe57": "tests/Fe57.json",
    "Fe58": "tests/Fe58.json",
    "Li6": "tests/Li6.json",
    "Li7": "tests/Li7.json",
})

nuc1 = m4mc.Nuclide('Li6')
print(nuc1)

nuc1.read_nuclide_from_json("tendl-21")
