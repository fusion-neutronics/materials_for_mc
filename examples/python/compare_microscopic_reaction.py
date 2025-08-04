import openmc
import materials_for_mc as m4mc
import math

openmc.config['cross_sections'] = '/home/jon/nuclear_data/cross_sections.xml'

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


import plotly.graph_objects as go
fig = go.Figure()
fig.add_trace(go.Scatter(x=openmc_energies, y=openmc_xs, mode='lines', name='OpenMC', line=dict(dash='dash')))
fig.add_trace(go.Scatter(x=energies, y=xs, mode='lines', name='My code', line=dict(dash='dot')))
fig.update_layout(
    title='Li6 Neutron Cross Section Comparison',
    xaxis_title='Energy (eV)',
    yaxis_title='Cross Section (barns)',
    legend=dict(x=0.01, y=0.99),
    xaxis_type='log',
    yaxis_type='log',
    template='plotly_white'
)
fig.write_html('compare_microscopic_reaction.html')