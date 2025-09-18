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

element1 = m4mc.Element('Li')

print(element1)


micro_n_gamma , energy = element1.microscopic_cross_section(reaction='(n,gamma)', temperature='294')
micro_mt_3 , energy = element1.microscopic_cross_section(reaction=3)

print(micro_n_gamma)