import openmc
import materials_for_mc as m4mc
import math

openmc_energies, openmc_xs = openmc.calculate_cexs('Li6', [2], temperature=294)
openmc_xs=openmc_xs[0]


nuc = m4mc.Nuclide()
nuc.read_nuclide_from_json('tests/Li6.json')
xs = nuc.reactions['294'][2]['cross_section']
energies = nuc.reactions['294'][2]['energy']

# nuc.microscopic_xs_neutron(temperature, mt)
# nuc.microscopic_xs_neutron[2]

for openmc_x, my_x in zip(openmc_xs, xs):
    print(f'OpenMC: {openmc_x}, My code: {my_x}')
    assert math.isclose(openmc_x , my_x, rel_tol=1e-6, abs_tol=1e-6)

for openmc_energy, energy in zip(openmc_energies, energies):
    print(f'OpenMC: {openmc_energy}, My code: {energy}')
    assert math.isclose(openmc_energy , energy, rel_tol=1e-6, abs_tol=1e-6)


import matplotlib.pyplot as plt
plt.plot(openmc_energies, openmc_xs, label='OpenMC', linestyle='--')
plt.plot(energies, xs, label='My code', linestyle='-.')  
plt.xlabel('Energy (eV)')
plt.ylabel('Cross Section (barns)')
plt.title('Li6 Neutron Cross Section Comparison')
plt.legend()
plt.xscale('log')
plt.yscale('log')
plt.grid(True)
plt.show()