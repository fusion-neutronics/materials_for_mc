import materials_for_mc as m4mc
m4mc.Config.set_cross_sections({
    "Li6": "https://raw.githubusercontent.com/fusion-neutronics/cross_section_data_tendl_2021/refs/heads/main/tendl_2021/Li6.json",
    "Li7": "tests/Li7.json"
})

mat1 = m4mc.Material()
mat1.add_element('Li', 0.5)
mat1.set_density('g/cm3', 2.0)
mat1.volume = 4.2
mat1.read_nuclides_from_json()
print(mat1)

nuc = m4mc.Nuclide('Li6')
nuc.read_nuclide_from_json("tests/Li6.json")

nuc = m4mc.Nuclide('Li7')
nuc.read_nuclide_from_json("https://raw.githubusercontent.com/fusion-neutronics/cross_section_data_tendl_2021/refs/heads/main/tendl_2021/Li7.json")
