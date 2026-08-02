# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

import os 
import sys
sys.path.insert(0, os.path.abspath('../../'))

project = 'batek'
copyright = '2026, yasosu'
author = 'yasosu'
release = '1'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = [
    'sphinx.ext.autodoc',       # Генерация документации из docstrings
    'sphinx.ext.napoleon',      # Поддержка Google и NumPy стиля docstrings
    'sphinx.ext.viewcode',      # Добавляет ссылки на исходный код
    'sphinx.ext.todo',          # Поддержка заметок TODO
]

autodoc_mock_imports = ["sqlalchemy", "asyncpg", "alembic", "discord", "dotenv", "yt_dlp"]


templates_path = ['_templates']
exclude_patterns = []



# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'alabaster'
html_static_path = ['_static']
