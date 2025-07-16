import numpy as np
import openmc
import materials_for_mc as m4mc

openmc.config['cross_sections'] = '/home/jon/nuclear_data/cross_sections.xml'
mat = openmc.Material()
mat.add_nuclide('Li6',1)
mat.set_density('g/cm3', 20.)


openmc_energies, openmc_xs = openmc.calculate_cexs(mat, [2], temperature=294)
openmc_xs=openmc_xs[0]


mat1 = m4mc.Material()
mat1.add_nuclide('Li6',1)
mat1.set_density('g/cm3',20.)

mat1.temperature = "294"  # Set temperature directly on the material
mat1.read_nuclides_from_json({'Li6':'tests/li6_neutron.json'})
mat1.calculate_macroscopic_xs_neutron()
my_macro = mat1.macroscopic_xs_neutron['2']
# Get the unified energy grid}')

import matplotlib.pyplot as plt
plt.plot(openmc_energies, openmc_xs, label='OpenMC', linestyle='--')
plt.plot(mat1.unified_energy_grid_neutron(),my_macro, label='My code', linestyle='-.')
plt.xlabel('Energy (eV)')
plt.ylabel('Cross Section (barns)')
plt.title('Li6 Neutron Macroscopic Cross Section Comparison')
plt.legend()
plt.xscale('log')
plt.yscale('log')
plt.grid(True)
plt.show()