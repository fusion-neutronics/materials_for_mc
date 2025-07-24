import numpy as np
import openmc
import materials_for_mc as m4mc

openmc.config['cross_sections'] = '/home/jon/nuclear_data/cross_sections.xml'
mat = openmc.Material()
mat.add_nuclide('Li6',1)
mat.set_density('g/cm3', 20.)


openmc_energies, openmc_xs = openmc.calculate_cexs(mat, [1], temperature=294)
openmc_xs=openmc_xs[0]


mat1 = m4mc.Material()
mat1.add_nuclide('Li6',1)
mat1.set_density('g/cm3',20.)

mat1.temperature = "294"  # Set temperature directly on the material
mat1.read_nuclides_from_json({'Li6':'tests/Li6.json'})
# mat1.calculate_macroscopic_xs_neutron()
mat1.calculate_total_xs_neutron()
my_macro = mat1.macroscopic_xs_neutron[1]
my_energies = mat1.unified_energy_grid_neutron()


for openmc_energy, my_energy in zip(openmc_energies, my_energies):
    # print(f'OpenMC: {openmc_energy}, My code: {my_energy}')
    assert np.isclose(openmc_energy , my_energy, rtol=1e-6, atol=1e-6)

# for openmc_x, my_x in zip(openmc_xs, my_macro):
    # print(f'OpenMC: {openmc_x}, My code: {my_x}')
    # assert np.isclose(openmc_x, my_x, rtol=1e-6, atol=1e-6)


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

import plotly.graph_objects as go
from plotly.subplots import make_subplots

# Create a figure with secondary y-axis
fig = go.Figure()

# Add trace for my code's total cross-section
fig.add_trace(
    go.Scatter(
        x=my_energies,
        y=my_macro,
        name='mycode (total)',
        line=dict(dash='dashdot', width=2)
    )
)

# Add traces for individual MT reaction cross-sections from my code
for mt in list(mat1.macroscopic_xs_neutron.keys()):
    my_macro = mat1.macroscopic_xs_neutron[mt]
    if my_macro[0] != 0 and mt != 1:  # Skip empty reactions and avoid duplicating total
        fig.add_trace(
            go.Scatter(
                x=my_energies,
                y=my_macro,
                name=f'MT={mt}',
                line=dict(dash='dashdot', width=1),
                visible='legendonly'  # Hide by default to reduce clutter
            )
        )

# Add trace for OpenMC total cross-section
fig.add_trace(
    go.Scatter(
        x=openmc_energies,
        y=openmc_xs,
        name='OpenMC (total)',
        line=dict(dash='dash', width=2)
    )
)

# Update layout with logarithmic scales and labels
fig.update_layout(
    title='Li6 Neutron Macroscopic Cross Section Comparison',
    xaxis_title='Energy (eV)',
    yaxis_title='Cross Section (barns)',
    xaxis_type='log',
    yaxis_type='log',
    legend=dict(
        yanchor="top",
        y=0.99,
        xanchor="left",
        x=0.01
    ),
    hovermode='closest'
)

fig.update_xaxes(gridcolor='lightgray')
fig.update_yaxes(gridcolor='lightgray')

# Show the figure
fig.write_html('macroscopic_total_comparison.html')

