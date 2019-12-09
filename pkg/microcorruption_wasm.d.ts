/* tslint:disable */
/**
* @param {string} name 
* @param {Uint8Array} memory 
* @param {any} symbols 
* @returns {Uint8Array} 
*/
export function gen_elf(name: string, memory: Uint8Array, symbols: any): Uint8Array;

/**
* If `module_or_path` is {RequestInfo}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {RequestInfo | BufferSource | WebAssembly.Module} module_or_path
*
* @returns {Promise<any>}
*/
export default function init (module_or_path: RequestInfo | BufferSource | WebAssembly.Module): Promise<any>;
        