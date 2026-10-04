"""Entry point: python -m tools.jenkins.gap <command>"""
import sys
from .cli import main

sys.exit(main())
