# Python Usage

The Python API allows you to make Nuclides, Elements, Materials and access their properties. If you have Configured the nuclear data then you can access macroscopic and microscopic cross sections as well.


## Quick start guide

In this quick start guide we
- import the package
- configure the nuclear data
- make a nuclide
- get the nuclide microscopic cross section
- make a material
- get the material macroscopic cross section

### Import

```python
import materials_for_mc as m4mc
```

### Config the nuclear data

For now we will specify a single nuclear data library to use for all nuclides.
This is the simplest option other config options are covered later.  

```python
m4mc.Config.set_cross_sections("fendl-3.2c")  # tendl-21 is another option
```

## Create a nuclides

Nuclides can be made and their basic properties accessed like this

```python
nuclide = m4mc.Nuclide('Li6')
```

The microscopic cross section for a specific reaction can then be found for and MT number with.

```python
xs, energy = nuclide.microscopic_cross_section(reaction="(n,gamma)")
```

## Creating a Material

A material can be made by adding elements or nuclides with their atom fractions.

```python
material = m4mc.Material()
material.add_element('Li', 0.5)
material.add_nuclide('B10', 0.5)
```

The density must also be set to complete the material.

```python
material.set_density('g/cm3', 7.1)  # kg/m3 also accepted
```

The macroscopic cross section for a specific reaction can then be found for and MT number with.

```python
xs, energy = material.macroscopic_cross_section(reaction="(n,total)")
```





## Setting nuclear data

User control over the source of nuclear data for each on a nuclide and material level is facilitated in a few ways.

### Config specific libraries

The simplest method is to configure the package to use a single source of nuclear data for all nuclides.

```python
m4mc.Config.set_cross_sections("tendl-21")
```

Whenever nuclear data is needed this will check the users ```.cache/materials_for_mc``` folder to see if the JSON file for the required nuclide exists.
If the file is found then it will be used and if not the JSON file will be downloaded to the cache and then used.

### Config with JSON paths

It is also possible to download JSON files from nuclear data repos [fendl-3.2](https://github.com/fusion-neutronics/cross_section_data_fendl_3.2c) and [tendl-21](https://github.com/fusion-neutronics/cross_section_data_tendl_21).Once the JSON files are saved locally then the path to these files can be used to configure the nuclear data. Replace the ```tests``` dir to the path to the downloaded JSON files.

```python
m4mc.Config.set_cross_sections({
    "Be9": "tests/Be9.json",
    "Fe54": "tests/Fe54.json",
    "Fe56": "tests/Fe56.json",
    "Fe57": "tests/Fe57.json",
    "Fe58": "tests/Fe58.json",
    "Li6": "tests/Li6.json",
    "Li7": "tests/Li7.json",
})
```

### Config with JSON paths and specific libraries

It is also possible to mix different sources when configuring the nuclear data sources. In this example we use tendl-21 for some nuclides, file paths for others and fendl-3.2c for others. Replace the ```tests``` dir to the path to the downloaded JSON files.

```python
m4mc.Config.set_cross_sections({
    "Be9": "tendl-21",
    "Fe54": "tendl-21",
    "Fe56": "tests/Fe56.json",
    "Fe57": "tests/Fe57.json",
    "Fe58": "tests/Fe58.json",
    "Li6": "fendl-3.2c/Li6.json",
    "Li7": "fendl-3.2c/Li7.json",
})
```

### Specific nuclear data on the Nuclide

You can also avoid the ```Config``` entirely and specify the nuclear data to use on the Nuclide object directly.

This can be done using a nuclear data library.
```python
nuclide = m4mc.Nuclide('Li6')
nuclide.read_nuclide_from_json('tendl-21')
```

Alternatively a specific nuclear data path

```python
nuclide = m4mc.Nuclide('Li6')
nuclide.read_nuclide_from_json('tests/Li6.json')
```

### Specific nuclear data on the Material

You can also specify the nuclear data on a material directly.
Again this can be done using a nuclear data library.

```python
material = m4mc.Material()
material.add_nuclide('Li6',1)
material.set_density('g/cm3',2.)
material.read_nuclides_from_json({'Li6':'tendl-21'})
material.temperature = "294"  # needed if there are multiple temperatures 
my_energies, xs_dict = material.calculate_macroscopic_xs([3])
```

Alternatively a specific nuclear data path

```python
material = m4mc.Material()
material.add_nuclide('Li6',1)
material.set_density('g/cm3',2.)
material.read_nuclides_from_json({'Li6':'tests/Li6.json'})
material.temperature = "294"  # needed if there are multiple temperatures 
my_energies, xs_dict = material.calculate_macroscopic_xs([3])
```


## Monte Carlo transport features

If building a Monte Carlo code on top of this package then it is recommended to use the Rust API to access the Monte Carlo specific properties as it offers a offers a speed advantage.  
However the Python API also provides access to all the Monte Carlo Transport properties such as mean free path, sampling interaction distance, interacting nuclide and interacting reaction.

### Mean free path

The mean free path of a neutron with a specified energy in a material can be found using ```Material.mean_free_path_neutron()```.

```python
import materials_for_mc as m4mc
mat1 = m4mc.Material()
mat1.add_nuclide('Li6',1)
mat1.add_nuclide('Li7',1)
mat1.set_density('g/cm3',1.)
mat1.temperature = "294"
mat1.read_nuclides_from_json({'Li6':'tests/Li6.json', 'Li7':'tests/Li7.json'})
mean_free_path = mat1.mean_free_path_neutron(14e6)
print(f'mean free path is {mean_free_path}')
```

### Sample distance to collision

The distance to the collision can be sampled for a given neutron energy using ``` Material.distance to the collision()```.
This is used by Monte Carlo codes to determine if an interaction occurs within the material.


The distance to a collision within a material can be sampled
```python
import materials_for_mc as m4mc
mat = m4mc.Material()
mat.add_nuclide("Li6", 1.0)
mat.set_density("g/cm3", 1.)
mat.read_nuclides_from_json({"Li6": "tests/Li6.json"})
mat.temperature = "294"
mat.calculate_macroscopic_xs()  # Ensure xs are calculated
sampled_distance = mat.sample_distance_to_collision(energy=14e6, seed=1234)
print(f'sampled interaction distance is {sampled_distance}')
```

### Sample interacting nuclide

If the sampled distance is less than the mean free path then an interaction happens.
Monte Carlo codes typically then sample the interacting nuclide.
The interacting nuclide can be sampled for a given neutron energy using ```material. sample_interacting_nuclide()```

```python
import materials_for_mc as m4mc
material = m4mc.Material()
material.add_nuclide("Li6", 0.5)
material.add_nuclide("Li7", 0.5)
material.set_density("g/cm3", 1.0)
material.temperature = "294"
material.read_nuclides_from_json({
    "Li6": "tests/Li6.json",
    "Li7": "tests/Li7.json",
})
# Calculate per-nuclide macroscopic total xs
material.calculate_macroscopic_xs(mt_filter=[1], by_nuclide=True)
interacting_nuclide = material.sample_interacting_nuclide(energy=2.5e6, seed=456)
print(f'interacting nuclide is {interacting_nuclide}')
```

### Sample interacting reaction

Once the interacting nuclide is known then the reaction can be sampled using ```Nuclide.sample_reaction()```.

```python
import materials_for_mc as m4mc
nuc = m4mc.Nuclide('Li6')
nuc.read_nuclide_from_json('tests/Li6.json')
reaction = nuc.sample_reaction(energy=1.0, temperature='294', seed=42)
print(f'interacting reaction is {reaction}')
```