#!/usr/bin/env python3
"""
Simple example comparing Fe56 materials with cross sections from different data sources.
Material-level loading now works correctly!
"""

import materials_for_mc as m4mc
import matplotlib.pyplot as plt


# Material 1: Li7 from TENDL-21
print("Creating Li7 material from TENDL-21...")
mat_tendl = m4mc.Material()
mat_tendl.add_nuclide("Li7", 1.0)
print("Reading Li7 from TENDL-21...")
mat_tendl.read_nuclides_from_json("tendl-21")
mat_tendl.set_density("g/cm3", 0.534)  # Lithium density

# Material 2: Li7 from FENDL-3.2c  
print("Creating Li7 material from FENDL-3.2c...")
mat_fendl = m4mc.Material()
mat_fendl.add_nuclide("Li7", 1.0)
print("Reading Li7 from FENDL-3.2c...")
mat_fendl.read_nuclides_from_json("fendl-3.2c")
mat_fendl.set_density("g/cm3", 0.534)  # Lithium density

# Get macroscopic (n,gamma) cross sections from both materials
print("Getting macroscopic (n,gamma) cross sections...")
xs_tendl, energy_tendl = mat_tendl.macroscopic_cross_section("(n,gamma)")
xs_fendl, energy_fendl = mat_fendl.macroscopic_cross_section("(n,gamma)")

# Plot comparison
plt.figure(figsize=(10, 6))
plt.loglog(energy_tendl, xs_tendl, label="TENDL-21", linewidth=1.5)
plt.loglog(energy_fendl, xs_fendl, label="FENDL-3.2c", linewidth=1.5, linestyle='--')

plt.xlabel("Energy (eV)")
plt.ylabel("Macroscopic Cross Section (cm⁻¹)")
plt.title("Fe56 Material (n,gamma) Macroscopic Cross Section Comparison")
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()

# Save and show
plt.savefig("Fe56_material_comparison.png", dpi=150, bbox_inches='tight')
print("Plot saved as Fe56_material_comparison.png")
plt.show()
