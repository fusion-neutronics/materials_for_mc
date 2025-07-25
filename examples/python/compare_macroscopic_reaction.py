import numpy as np
import openmc
import materials_for_mc as m4mc

openmc.config['cross_sections'] = '/home/jon/nuclear_data/cross_sections.xml'
mat1 = openmc.Material()
mat1.add_nuclide('Li6',1)
mat1.set_density('g/cm3', 2.)
openmc_energies, openmc_xs = openmc.calculate_cexs(mat1, [3], temperature=294)
openmc_macro=openmc_xs[0]


mat2 = m4mc.Material()
mat2.add_nuclide('Li6',1)
mat2.read_nuclides_from_json({'Li6':'tests/Li6.json'})
mat2.set_density('g/cm3',2.)
mat2.temperature = "294"
my_energies, xs_dict = mat2.calculate_macroscopic_xs_neutron([3])
my_macro = xs_dict[3]

import matplotlib.pyplot as plt
plt.plot(openmc_energies, openmc_macro, label='OpenMC', linestyle='--')
plt.plot(my_energies,my_macro, label='My code', linestyle='-.')
plt.xlabel('Energy (eV)')
plt.ylabel('Cross Section (barns)')
plt.title('Li6 Neutron Macroscopic Cross Section Comparison')
plt.legend()
plt.xscale('log')
plt.yscale('log')
plt.grid(True)
plt.show()