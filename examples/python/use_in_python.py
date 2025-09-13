import materials_for_mc as m4mc
# m4mc.Config.set_cross_sections({
#     "Li6": "tendl-21",
#     "Li7": "tendl-21"
# })
m4mc.Config.set_cross_section("tendl-21")

mat1 = m4mc.Material()
mat1.add_element('Li', 0.5)
mat1.set_density('g/cm3', 2.0)
mat1.volume = 4.2
mat1.read_nuclides_from_json()
print(mat1)
