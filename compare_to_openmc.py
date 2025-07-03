import numpy as np
import openmc

openmc.config['cross_sections'] = '/home/jon/nuclear_data/cross_sections.xml'
mat = openmc.Material()
mat.add_nuclide('Li6',1)
mat.set_density('g/cm3', 1.)

openmc_macroscopic_xs_total_at_14mev = 1/mat.mean_free_path(14e6)


import materials_for_mc as m4mc

mat1 = m4mc.Material()
mat1.add_nuclide('Li6',1)
mat1.set_density('g/cm3',1.)

mat1.temperature = "294"  # Set temperature directly on the material
mat1.read_nuclides_from_json({'Li6':'tests/li6_neutron.json'})


# Get the unified energy grid
grid = mat1.unified_energy_grid_neutron()
total_xs = mat1.calculate_total_xs_neutron()

my_code_macro_total_at_14mev = float(np.interp(14e6, grid, total_xs['total']))

print('openmc                           ' , openmc_macroscopic_xs_total_at_14mev)
print('my code macroscopic XS at 14 MeV:', my_code_macro_total_at_14mev)