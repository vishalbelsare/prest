#!/usr/bin/env python3
#
# Prest documentation build configuration file, created by
# sphinx-quickstart on Mon Mar  5 10:18:47 2018.
#
# This file is execfile()d with the current directory set to its
# containing dir.
#
# Note that not all possible configuration values are present in this
# autogenerated file.
#
# All configuration values have a default; values that are commented out
# serve to show the default.

# If extensions (or modules to document with autodoc) are in another directory,
# add these directories to sys.path here. If the directory is relative to the
# documentation root, use os.path.abspath to make it absolute, like shown here.
#
import os
import sys
sys.path.insert(0, os.path.abspath('extensions'))


# -- General configuration ------------------------------------------------

# If your documentation needs a minimal Sphinx version, state it here.
#
# needs_sphinx = '1.0'

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    'sphinx.ext.todo',
    #'sphinx.ext.imgmath',  ## TODO: enable this for production
    'sphinx.ext.mathjax',
    'sphinx.ext.ifconfig',
    'sphinxcontrib.bibtex',
    'sphinx.ext.autosectionlabel',
    'google_analytics',
]

#bibtex_bibliography_header = ".. rubric:: References"
#bibtex_footbibliography_header = ""
bibtex_bibfiles = ['references.bib']
googleanalytics_id = 'UA-125271613-1'

bibtex_encoding = 'utf-8-sig'
#bibtex_default_style = 'plain'

#mathjax_path = 'https://cdnjs.cloudflare.com/ajax/libs/mathjax/2.7.5/MathJax.js?config=TeX-MML-AM_CHTML'
#mathjax_path = 'https://prestsoftware.com/_static/mathjax/MathJax.js?config=TeX-MML-AM_CHTML'
mathjax_path = 'https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-svg.js'

# For DOI external links

extlinks = {
    'doi': ('https://dx.doi.org/%s', 'doi:'),
}

# Add any paths that contain templates here, relative to this directory.
templates_path = ['_templates']

# The suffix(es) of source filenames.
# You can specify multiple suffix as a list of string:
#
# source_suffix = ['.rst', '.md']
source_suffix = '.rst'

# The master toctree document.
master_doc = 'index'

# General information about the project.
project = 'Prest'
copyright = '2018-2022, Georgios Gerasimou, Matúš Tejiščák'
author = 'Georgios Gerasimou, Matúš Tejiščák'

# The version info for the project you're documenting, acts as replacement for
# |version| and |release|, also used in various other places throughout the
# built documents.
#
# The short X.Y version.
try:
    with open('../doc-version.txt') as f:
        version = f.read().strip()
except OSError:
#    version = '(no version)'
    version = ''
# The full version, including alpha/beta/rc tags.
release = version

# The language for content autogenerated by Sphinx. Refer to documentation
# for a list of supported languages.
#
# This is also used if you do content translation via gettext catalogs.
# Usually you set "language" from the command line for these cases.
language = None

# List of patterns, relative to source directory, that match files and
# directories to ignore when looking for source files.
# This patterns also effect to html_static_path and html_extra_path
exclude_patterns = []

# The name of the Pygments (syntax highlighting) style to use.
pygments_style = 'sphinx'

# If true, `todo` and `todoList` produce output, else they produce nothing.
todo_include_todos = True


# -- Options for HTML output ----------------------------------------------

# The theme to use for HTML and HTML Help pages.  See the documentation for
# a list of builtin themes.

extensions.append("sphinxjp.themes.basicstrap")
html_theme = 'basicstrap'


# Theme options are theme-specific and customize the look and feel of a theme
# further.  For a list of options available for each theme, see the
# documentation.
#
html_theme_options = {

    # Set the lang attribute of the html tag. Defaults to 'en'
    'lang': 'en',
    # Disable showing the sidebar. Defaults to 'false'
    'nosidebar': False,
    # Show header searchbox. Defaults to false. works only "nosidebar=True",
    'header_searchbox': False,

    # Put the sidebar on the right side. Defaults to false.
    'rightsidebar': False,
    # Set the width of the sidebar. Defaults to 3
    'sidebar_span': 3,

    # Fix navbar to top of screen. Defaults to true
    'nav_fixed_top': False,
    # Fix the width of the sidebar. Defaults to false
    'nav_fixed': False,
    # Set the width of the sidebar. Defaults to '900px'
    'nav_width': '900px',
    # Fix the width of the content area. Defaults to false
    'content_fixed': False,
    # Set the width of the content area. Defaults to '900px'
    'content_width': '900px',
    # Fix the width of the row. Defaults to false
    'row_fixed': False,

    # Disable the responsive design. Defaults to false
    'noresponsive': False,
    # Disable the responsive footer relbar. Defaults to false
    'noresponsiverelbar': False,
    # Disable flat design. Defaults to false.
    # Works only "bootstrap_version = 3"
    'noflatdesign': False,

    # Enable Google Web Font. Defaults to false
    'googlewebfont': False,
    # Set the URL of Google Web Font's CSS.
    # Defaults to 'http://fonts.googleapis.com/css?family=Text+Me+One'
    'googlewebfont_url': 'http://fonts.googleapis.com/css?family=Lily+Script+One',  # NOQA
    # Set the Style of Google Web Font's CSS.
    # Defaults to "font-family: 'Text Me One', sans-serif;"
    'googlewebfont_style': u"font-family: 'Lily Script One' cursive;",

    # Set 'navbar-inverse' attribute to header navbar. Defaults to false.
    'header_inverse': False,
    # Set 'navbar-inverse' attribute to relbar navbar. Defaults to false.
    'relbar_inverse': False,

    # Enable inner theme by Bootswatch. Defaults to false
    'inner_theme': True,
    # Set the name of innner theme. Defaults to 'bootswatch-simplex'
    #'inner_theme_name': 'bootswatch-flatly',
    'inner_theme_name': 'bootswatch-cerulean',

    # Select Twitter bootstrap version 2 or 3. Defaults to '3'
    'bootstrap_version': '3',

    # Show "theme preview" button in header navbar. Defaults to false.
    'theme_preview': False,

    # Set the Size of Heading text. Defaults to None
     #'h1_size': '3.0em',
     #'h2_size': '2.6em',
     #'h3_size': '2.2em',
     #'h4_size': '1.8em',
     #'h5_size': '1.4em',
     #'h6_size': '1.1em',
}

# Add any paths that contain custom static files (such as style sheets) here,
# relative to this directory. They are copied after the builtin static files,
# so a file named "default.css" will overwrite the builtin "default.css".
html_static_path = ['_static']
html_favicon = '_static/favicon.png'


#TEXT-WIDTH CUSTOMISATION
#def setup(app):
#    app.add_css_file('my_theme.css')

# Custom sidebar templates, must be a dictionary that maps document names
# to template names.
#
# This is required for the alabaster theme
# refs: http://alabaster.readthedocs.io/en/latest/installation.html#sidebars
html_sidebars = {
    '**': [
        'globaltoc.html',
#        'localtoc.html',
#        'relations.html',  # needs 'show_related': True theme option to display
#        'searchbox.html',
    ]
}


# -- Options for HTMLHelp output ------------------------------------------

# Output file base name for HTML help builder.
htmlhelp_basename = 'Prestdoc'


# -- Options for LaTeX output ---------------------------------------------

latex_elements = {
    # The paper size ('letterpaper' or 'a4paper').
    #
    # 'papersize': 'letterpaper',

    # The font size ('10pt', '11pt' or '12pt').
    #
    # 'pointsize': '10pt',

    # Additional stuff for the LaTeX preamble.
    #
    # 'preamble': '',

    # Latex figure (float) alignment
    #
    # 'figure_align': 'htbp',
}

# Grouping the document tree into LaTeX files. List of tuples
# (source start file, target name, title,
#  author, documentclass [howto, manual, or own class]).
latex_documents = [
    (master_doc, 'Prest.tex', 'Prest Documentation',
     'Georgios Gerasimou, Matúš Tejiščák', 'manual'),
]


# -- Options for manual page output ---------------------------------------

# One entry per manual page. List of tuples
# (source start file, name, description, authors, manual section).
man_pages = [
    (master_doc, 'prest', 'Prest Documentation',
     [author], 1)
]


# -- Options for Texinfo output -------------------------------------------

# Grouping the document tree into Texinfo files. List of tuples
# (source start file, target name, title, author,
#  dir menu entry, description, category)
texinfo_documents = [
    (master_doc, 'Prest', 'Prest Documentation',
     author, 'Prest', 'One line description of project.',
     'Miscellaneous'),
]

default_role = 'math'
imgmath_image_format = 'svg'
imgmath_latex_preamble = r'''
\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{amsopn}
\usepackage{amsthm}
\usepackage{latexsym}
\usepackage{setspace}
\usepackage{color}
\usepackage{microtype}
\usepackage[bookmarks=true,bookmarksnumbered=true,pdfpagemode=None,pdfstartview=FitH,hidelinks]{hyperref}

\newcommand{\R}{\mathbb{R}}

\parindent=0pt
\parskip=2pt
'''
