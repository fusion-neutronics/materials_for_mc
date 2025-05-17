import materials_for_mc
mat1 = materials_for_mc.Material()
mat1.add_nuclide('Li6', 0.5)
mat1.set_density('g/cm3', 2.0)
mat1.volume = 4.2
print(mat1)