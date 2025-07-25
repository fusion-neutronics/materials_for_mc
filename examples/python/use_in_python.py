import materials_for_mc as m4mc
m4mc.Config.set_cross_sections({
    "Li6": "tests/Li6.json",
    "Li7": "tests/Li7.json"
})

mat1 = m4mc.Material()
mat1.add_nuclide('Li6', 0.5)
mat1.set_density('g/cm3', 2.0)
mat1.volume = 4.2
print(mat1)

nuc = m4mc.Nuclide('Li6')
nuc.read_nuclide_from_json('tests/Li6.json')
nuc.reaction_mts
assert nuc.reaction_mts == [102, 103, 105, 2, 203, 204, 205, 207, 24, 301, 444, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81]


nuc = m4mc.Nuclide('Li6')
nuc.read_nuclide_from_json('tests/Li6.json')
nuc.reaction_mts