import numpy as np
import openmc

openmc.config['cross_sections'] = '/home/jon/nuclear_data/cross_sections.xml'
mat = openmc.Material()
mat.add_nuclide('Li6',1)
mat.set_density('g/cm3', 1.)

openmc_mean_free_path_at_14mev = mat.mean_free_path(14e6)

import materials_for_mc as m4mc

mat1 = m4mc.Material()
mat1.add_nuclide('Li6',1)
mat1.set_density('g/cm3',1.)
mat1.temperature = "294"
mat1.read_nuclides_from_json({'Li6':'tests/Li6.json'})

# Get the unified energy grid
m4mc_mean_free_path_at_14mev = mat1.mean_free_path_neutron(14e6)

print(f'My code mean free path at 14 MeV: {m4mc_mean_free_path_at_14mev}')
print(f'OpenMC mean free path at 14 MeV: {openmc_mean_free_path_at_14mev}')