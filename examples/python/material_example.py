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

mat1 = m4mc.Material()
mat1.add_nuclide('Li6', 0.5)
mat1.add_nuclide('Li7', 0.5)
mat1.add_nuclide('Fe56', 1.0)
mat1.add_nuclide('Be9', 1.0)
mat1.set_density('g/cm3', 2.0)
print(mat1)

# Demonstrate partial override: provide only one mapping (others pulled from Config)
# mat1.read_nuclides_from_json({"Li7": "tests/Li7.json"})

xs, energy = mat1.calculate_macroscopic_xs_neutron()
print(xs)