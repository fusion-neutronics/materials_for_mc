import openmc

openmc_energy, openmc_xs = openmc.calculate_cexs('Li6', [2], temperature=294)
openmc_xs=openmc_xs[0]

print(openmc_xs)

import materials_for_mc as m4mc

nuc = m4mc.Nuclide()
nuc.read_nuclide_from_json('tests/li6_neutron.json')
xs = nuc.incident_particle['neutron']['294']['2']['cross_section']
energy = nuc.energy_grid('294')

for openmc_x, my_x in zip(openmc_xs, xs):
    print(f'OpenMC: {openmc_x}, My code: {my_x}')