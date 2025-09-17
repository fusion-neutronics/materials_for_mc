# Python Usage

The Python API allows you to make Nuclides, Elements, Materials and access their properties. If you have Configured the nuclear data then you can access macroscopic and microscopic cross sections as well.


### Import

```python
import materials_for_mc as m4mc
```

### Config the nuclear data

For now we will specify a single nuclear data library to use for all nuclides.
This is the simplest option other config options are covered later.  

```python
m4mc.Config.set_cross_sections("fendl-3.2c")
```

## Nuclides

Nuclides can be made and their basic properties accessed like this

```python
nuclide = m4mc.Nuclide('Li6')
```

The microscopic cross section for a specific reaction can then be found for and MT number with.

```python
xs, energy = nuclide.microscopic_cross_section(1)
```

## Elements

Elements can be made and their basic properties accessed like this

```python
element = m4mc.Nuclide('Li')
```

### Element microscopic cross section 

## Creating a Material



## Setting nuclear data




### Config specify JSON paths

### Config specify combinations of nuclear data libraries and JSON paths

If building a Monte Carlo code on top of this package then it is recommended to use the Rust API to access the Monte Carlo specific properties as it offers a offers a speed advantage.  
However the Python API provides access to all the Monte Carlo Transport properties such as mean free path, sampling of interacting nuclide and interacting reaction for completeness.


## Monte Carlo transport features

### Mean free path

### Sample nuclide

### Sample reaction