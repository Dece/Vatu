#!/usr/bin/env python3

for f in range(8):
    for r in range(8):
        print("pub const {}{}: Pos = {};".format(chr(f + 65), r + 1, f * 8 + r))
