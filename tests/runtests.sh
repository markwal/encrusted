#!/bin/bash
cd "$(dirname "$0")"

# Unit tests
python regtest.py -i "../target/debug/encrusted" czech.z3.regtest
python regtest.py -i "../target/debug/encrusted" czech.z4.regtest
python regtest.py -i "../target/debug/encrusted" czech.z5.regtest
python regtest.py -i "../target/debug/encrusted" czech.z8.regtest
python regtest.py -i "../target/debug/encrusted" praxix.z5.regtest

# Game tests
python regtest.py -i "../target/debug/encrusted" curses.z3.regtest
python regtest.py -i "../target/debug/encrusted" minizork.z3.regtest
