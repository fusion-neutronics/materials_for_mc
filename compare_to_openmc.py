import openmc

openmc.config['cross_sections'] = '/home/jon/nuclear_data/cross_sections.xml'
mat = openmc.Material()
mat.add_nuclide('Li6',1)
mat.set_density('g/cm3', 2.)

print(mat.mean_free_path(14e6))

import materials_for_mc as m4mc

mat1 = m4mc.Material()
mat1.add_nuclide('Li6',1)
mat1.set_density('g/cm3',2.)

mat1.read_nuclides_from_json({'Li6':'tests/li6.json'})
mat1.temperature = "294"  # Set temperature directly on the material

# Get the unified energy grid
grid = mat1.unified_energy_grid_neutron()

# Calculate microscopic cross sections for all MT numbers
micro_xs = mat1.calculate_microscopic_xs_neutron(grid)
import numpy as np

micro_xs_at_14 = float(np.interp(14e6, grid, micro_xs))



mean_free_path = 1/micro_xs_at_14
print(mean_free_path)