// ==UserScript==
// @name        Memory Dump for Microcorruption
// @namespace   Kitlith
// @match       https://*.microcorruption.com/cpu/debugger
// @grant       GM_getResourceURL
// @version     0.0.3
// @author      Kitlith
// @require     https://cdn.jsdelivr.net/npm/file-saver@2.0.2/dist/FileSaver.min.js
// @require     microcorruption_wasm.js
// @resource    wasm microcorruption_wasm_bg.wasm
// @description Adds a command (dump) to the debugger that performs a memory dump (w/ symbols!) and downloads it as "<level name>.elf"
// ==/UserScript==

// TODO: do something that doesn't require granting an API to avoid unsafeWindow
// For now, this works without too much modification.
wasm_bindgen(GM_getResourceURL('wasm'));

function getMemory() {
    // cpu.memory is a sparse js 'array', let's convert it to a full 64KiB array before downloading
    var memory = new Uint8Array(0x10000);
    for (key in cpu.memory) {
        memory[key] = cpu.memory[key];
    }

    return memory;
}

unsafeWindow.cpu._dump = function (e) {
    var symbols = {};
    for (elem of document.getElementsByClassName("insnlabel")) {
        let addr = parseInt(elem.innerText.slice(0, 4), 16); // first four characters are address in hex
        if (isNaN(addr)) {
            continue;
        }
        let name = elem.textContent.slice(7, -2); // skip the addr, skip ' <', and leave out '>'
        symbols[name] = addr;
    }

    // By querying whoami, we can get the current level name. woo!
    cpu.get('/whoami', function(e) {
        var memory = getMemory();
        let elf = wasm_bindgen.gen_elf(e.level, memory, symbols);
        saveAs(new Blob([elf], {type: "application/octet-stream"}), e.level + ".elf");
    });
};

unsafeWindow.cpu._dumpbin = function () {
    cpu.get('/whoami', function(e) {
        var memory = getMemory();
        saveAs(new Blob([memory], {type: "application/octet-stream"}), e.level + ".bin");
    });
}
