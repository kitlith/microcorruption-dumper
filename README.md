# Microcorruption Level Dumper
_Liberate yourself from the microcorruption website_

## Usage
You'll need a userscript manager to use this.
This userscript was tested with Violentmonkey.
Grab the userscript from [jsDelivr](https://cdn.jsdelivr.net/gh/kitlith/microcorruption-dumper/pkg/mcorrupt.user.js).

To dump a microcorruption level, type `dump` in the on-page debugger.
This script will perform a memory dump and parse the page for symbols,
providing an ELF file suitable for viewing in Ghidra for download.

Additionally, the `dumpbin` command just performs a memory dump and downloads
it, for use with tools like
[msp430-emu-uctf](https://github.com/cemeyer/msp430-emu-uctf) that don't
support the elf file format.


## Hacking
Build the wasm component with:
```sh
wasm-pack build -t no-modules
```

To ease development, you may want to run a local web server hosting
the `pkg` directory. A sample node.js server has been included as serve.js.

The wasm component is written in Rust, and handles the generation of the ELF
file from the information gathered by the userscript driver located
at `pkg/mcorrupt.user.js`.
