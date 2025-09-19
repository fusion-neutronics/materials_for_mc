#!/usr/bin/env python3
"""
Simple example comparing Fe56 materials with cross sections from different data sources.
"""

import materials_for_mc as m4mc
import matplotlib.pyplot as plt


# Material 1: Fe56 from TENDL-21
print("Creating Fe56 material from TENDL-21...")
mat_tendl = m4mc.Material()
mat_tendl.add_nuclide("Fe56", 1.0)
mat_tendl.read_nuclides_from_json("tendl-21")
mat_tendl.set_density("g/cm3", 7.87)  # Iron density

# Material 2: Fe56 from FENDL-3.2c  
print("Creating Fe56 material from FENDL-3.2c...")
mat_fendl = m4mc.Material()
mat_fendl.add_nuclide("Fe56", 1.0)
mat_fendl.read_nuclides_from_json("fendl-3.2c")
mat_fendl.set_density("g/cm3", 7.87)  # Iron density

# Get macroscopic (n,gamma) cross sections from both materials
print("Getting macroscopic (n,gamma) cross sections...")
xs_tendl, energy_tendl = mat_tendl.macroscopic_cross_section("(n,gamma)")
xs_fendl, energy_fendl = mat_fendl.macroscopic_cross_section("(n,gamma)")

# Plot comparison
print("Plotting comparison...")
plt.figure(figsize=(10, 6))
plt.loglog(xs_tendl, energy_tendl, label="TENDL-21", linewidth=1.5)
plt.loglog(xs_fendl, energy_fendl, label="FENDL-3.2c", linewidth=1.5, linestyle='--')

plt.xlabel("Energy (eV)")
plt.ylabel("Macroscopic Cross Section (cm⁻¹)")
plt.title("Fe56 Material (n,gamma) Macroscopic Cross Section Comparison")
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()

# Save and show
plt.savefig("fe56_material_comparison.png", dpi=150, bbox_inches='tight')
print("Plot saved as fe56_material_comparison.png")
plt.show()
