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
my_energies, xs_dict = mat2.calculate_macroscopic_xs([3])
my_macro = xs_dict[3]

import plotly.graph_objects as go
fig = go.Figure()
fig.add_trace(go.Scatter(x=openmc_energies, y=openmc_macro, mode='lines', name='OpenMC', line=dict(dash='dash')))
fig.add_trace(go.Scatter(x=my_energies, y=my_macro, mode='lines', name='My code', line=dict(dash='dot')))
fig.update_layout(
    title='Li6 Neutron Macroscopic Cross Section Comparison',
    xaxis_title='Energy (eV)',
    yaxis_title='Cross Section (barns)',
    legend=dict(x=0.01, y=0.99),
    xaxis_type='log',
    yaxis_type='log',
    template='plotly_white'
)
fig.write_html('compare_macroscopic_reaction.html')