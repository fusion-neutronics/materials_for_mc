import os, sys, importlib
sys.path.insert(0, os.path.abspath('../..'))

project = 'materials_for_mc'
copyright = '2025, fusion-neutronics'
author = 'fusion-neutronics'

# The full version, including alpha/beta/rc tags
release = '0.1.0'


# -- General configuration ---------------------------------------------------

# Add any Sphinx extension modules here, as strings.
extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.napoleon',
    'myst_parser',
]

# Setup autodoc mock imports if the compiled extension isn't available
autodoc_mock_imports = []
try:
    import materials_for_mc
except ImportError:
    autodoc_mock_imports.append('materials_for_mc')

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This pattern also affects html_static_path and html_extra_path.
exclude_patterns = []

# Source file parsers
source_suffix = {'.md': 'markdown'}

# MyST Parser configuration
myst_enable_extensions = [
    "colon_fence",      # Enable ::: fences for directives
    "deflist",          # Enable definition lists
    "dollarmath",       # Enable $$math$$ for math rendering
    "fieldlist",        # Enable field lists
    "html_admonition",  # Enable HTML admonition syntax
    "html_image",       # Enable HTML image syntax
    "replacements",     # Enable text replacements
    "smartquotes",      # Enable smart quotes
    "tasklist",         # Enable task lists
]


# -- Options for HTML output -------------------------------------------------

"""Sphinx configuration for materials_for_mc docs.

Uses the pydata-sphinx-theme when available for a clean, modern layout.
Falls back to the classic theme if the dependency is missing (helpful for
minimal environments). Custom CSS (custom.css) still applies on top.
"""

# Prefer pydata-sphinx-theme; fall back gracefully if unavailable.
if importlib.util.find_spec('pydata_sphinx_theme') is not None:
    html_theme = 'pydata_sphinx_theme'
    html_theme_options = {
        # Basic branding / nav controls
        "show_prev_next": False,
        "navigation_depth": 2,
        "show_toc_level": 2,
        # Top navbar behavior
        "navbar_center": ["navbar-nav"],
        # Color / style toggles (leave light/dark to theme defaults)
        "icon_links": [
            {
                "name": "GitHub",
                "url": "https://github.com/fusion-neutronics/materials_for_mc",
                "icon": "fa-brands fa-github",
                "type": "fontawesome",
            },
        ],
    }
else:
    html_theme = 'classic'

html_title = 'materials_for_mc API'

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']
html_css_files = ['custom.css']

# Configure autodoc
autodoc_member_order = 'bysource'
autodoc_default_options = {
    'members': True,
    'undoc-members': False,
    'show-inheritance': True,
    'special-members': '__init__,__new__',
    'private-members': False,
}

# Use stub files for documentation
autodoc_typehints = 'description'
autodoc_typehints_format = 'short'

# Enable autosummary for stub-based documentation
autosummary_generate = False

# Configure Napoleon for Google-style docstrings
napoleon_google_docstring = True
napoleon_numpy_docstring = False
napoleon_include_init_with_doc = True
napoleon_include_private_with_doc = False
napoleon_include_special_with_doc = True

# No rustdoc configuration needed - we'll link directly to the HTML files
