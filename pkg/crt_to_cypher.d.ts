/* tslint:disable */
/* eslint-disable */
export function parse_content(content: string): any;
export function get_node_count(content: string): number;
export function get_relationship_count(content: string): number;
export class Relationship {
  free(): void;
  constructor(from: string, to: string, rel_type: string);
  from: string;
  to: string;
  rel_type: string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_relationship_free: (a: number, b: number) => void;
  readonly relationship_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly relationship_from: (a: number) => [number, number];
  readonly relationship_to: (a: number) => [number, number];
  readonly relationship_rel_type: (a: number) => [number, number];
  readonly relationship_set_from: (a: number, b: number, c: number) => void;
  readonly relationship_set_to: (a: number, b: number, c: number) => void;
  readonly relationship_set_rel_type: (a: number, b: number, c: number) => void;
  readonly parse_content: (a: number, b: number) => [number, number, number];
  readonly get_node_count: (a: number, b: number) => number;
  readonly get_relationship_count: (a: number, b: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
