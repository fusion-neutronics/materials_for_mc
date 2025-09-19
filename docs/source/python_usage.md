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

### Config specify JSON paths

It is also possible to download JSON files from nuclear data repos [fendl-3.2](https://github.com/fusion-neutronics/cross_section_data_fendl_3.2c) and [tendl-21](https://github.com/fusion-neutronics/cross_section_data_tendl_21).Once the JSON files are saved locally then the path to these files can be used to configure the nuclear data.

```python
m4mc.Config.set_cross_sections({
    "Be9": "path-to-downloaded-repo/Be9.json",
    "Fe54": "path-to-downloaded-repo/Fe54.json",
    "Fe56": "path-to-downloaded-repo/Fe56.json",
    "Fe57": "path-to-downloaded-repo/Fe57.json",
    "Fe58": "path-to-downloaded-repo/Fe58.json",
    "Li6": "path-to-downloaded-repo/Li6.json",
    "Li7": "path-to-downloaded-repo/Li7.json",
})
```

### Config specify JSON paths and specific libraries

It is also possible to mix different sources when configuring the nuclear data sources. In this example we use tendl-21 for some nuclides, file paths for others and fendl-3.2c for others.

```python
m4mc.Config.set_cross_sections({
    "Be9": "tendl-21",
    "Fe54": "tendl-21",
    "Fe56": "path-to-downloaded-repo/Fe56.json",
    "Fe57": "path-to-downloaded-repo/Fe57.json",
    "Fe58": "path-to-downloaded-repo/Fe58.json",
    "Li6": "fendl-3.2c/Li6.json",
    "Li7": "fendl-3.2c/Li7.json",
})
```

### Specific nuclear data on the Nuclide

You can also avoid the ```Config``` entirely and specify the nuclear data to use on the Nuclide object its self.

```python
nuclide = m4mc.Nuclide('Li6')
nuclide.read_nuclide_from_json('tests/Li6.json')
```

### Specific nuclear data on the Material

### Config specify combinations of nuclear data libraries and JSON paths

If building a Monte Carlo code on top of this package then it is recommended to use the Rust API to access the Monte Carlo specific properties as it offers a offers a speed advantage.  
However the Python API provides access to all the Monte Carlo Transport properties such as mean free path, sampling of interacting nuclide and interacting reaction for completeness.


## Monte Carlo transport features

### Mean free path

### Sample nuclide

### Sample reaction