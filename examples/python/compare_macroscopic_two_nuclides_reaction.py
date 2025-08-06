import numpy as np
import openmc
import materials_for_mc as m4mc

openmc.config['cross_sections'] = '/home/jon/nuclear_data/cross_sections.xml'
mat = openmc.Material()
mat.add_element('Li', 1.0)
mat.set_density('g/cm3', 20.)


openmc_energies, openmc_xs = openmc.calculate_cexs(mat, [2], temperature=294)
openmc_xs=openmc_xs[0]


m4mc.Config.set_cross_sections({'Li6':'tests/Li6.json', 'Li7':'tests/Li7.json'})
mat1 = m4mc.Material()
mat1.add_element('lithium', 1.0)
mat1.set_density('g/cm3',20.)

mat1.temperature = "294"  # Set temperature directly on the material

mat1.calculate_macroscopic_xs_neutron(mt_filter=[2])  # returns energy, cross section pairs
my_macro = mat1.macroscopic_xs_neutron[2]
# Get the unified energy grid}')

import plotly.graph_objects as go
fig = go.Figure()
fig.add_trace(go.Scatter(x=openmc_energies, y=openmc_xs, mode='lines', name='OpenMC', line=dict(dash='dash')))
fig.add_trace(go.Scatter(x=mat1.unified_energy_grid_neutron(), y=my_macro, mode='lines', name='My code', line=dict(dash='dot')))
fig.update_layout(
    title='Li6 Neutron Macroscopic Cross Section Comparison',
    xaxis_title='Energy (eV)',
    yaxis_title='Cross Section (barns)',
    legend=dict(x=0.01, y=0.99),
    xaxis_type='log',
    yaxis_type='log',
    template='plotly_white'
)
fig.write_html('compare_macroscopic_two_nuclides_reaction.html')