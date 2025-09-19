#!/usr/bin/env python3
"""
Simple example comparing Fe56 cross sections from different data sources.
"""

import materials_for_mc as m4mc
import matplotlib.pyplot as plt


# Load Fe56 from TENDL-21
print("Loading Fe56 from TENDL-21...")
fe56_tendl = m4mc.Nuclide("Fe56")
fe56_tendl.read_nuclide_from_json("tendl-21")

# Load Fe56 from FENDL-3.2c  
print("Loading Fe56 from FENDL-3.2c...")
fe56_fendl = m4mc.Nuclide("Fe56")
fe56_fendl.read_nuclide_from_json("fendl-3.2c")

# Get (n,gamma) cross sections from both
xs_tendl, energy_tendl = fe56_tendl.microscopic_cross_section("(n,gamma)", "294")
xs_fendl, energy_fendl = fe56_fendl.microscopic_cross_section("(n,gamma)", "294")

# Plot comparison
plt.figure(figsize=(10, 6))
plt.loglog(energy_tendl, xs_tendl, label="TENDL-21", linewidth=1.5)
plt.loglog(energy_fendl, xs_fendl, label="FENDL-3.2c", linewidth=1.5, linestyle='--')

plt.xlabel("Energy (eV)")
plt.ylabel("Cross Section (barns)")
plt.title("Fe56 (n,gamma) Cross Section Comparison")
plt.legend()
plt.grid(True, alpha=0.3)
plt.tight_layout()

# Save and show
plt.savefig("fe56_comparison.png", dpi=150, bbox_inches='tight')
print("Plot saved as fe56_comparison.png")
