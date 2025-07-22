import materials_for_mc as m4mc
m4mc.Config.set_cross_sections({
    "Li6": "tests/li6.json",
    "Li7": "tests/Li7.json"
})

mat1 = m4mc.Material()
mat1.add_nuclide('Li6', 0.5)
mat1.set_density('g/cm3', 2.0)
mat1.volume = 4.2
print(mat1)