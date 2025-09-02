import { NodeApi } from "./interfaces/node-api";
import { TreeApi } from "./interfaces/tree-api";
import { IdObj } from "./types/utils";
export declare function bound(n: number, min: number, max: number): number;
export declare function isItem(node: NodeApi<any> | null): boolean | null;
export declare function isClosed(node: NodeApi<any> | null): boolean | null;
export declare function isOpenWithEmptyChildren(node: NodeApi<any> | null): boolean | null;
/**
 * Is first param a descendant of the second param
 */
export declare const isDescendant: (a: NodeApi<any>, b: NodeApi<any>) => boolean;
export declare const indexOf: (node: NodeApi<any>) => number;
export declare function noop(): void;
export declare function dfs(node: NodeApi<any>, id: string): NodeApi<any> | null;
export declare function walk(node: NodeApi<any>, fn: (node: NodeApi<any>) => void): void;
export declare function focusNextElement(target: HTMLElement): void;
export declare function focusPrevElement(target: HTMLElement): void;
export declare function access<T = boolean>(obj: any, accessor: string | boolean | Function): T;
export declare function identifyNull(obj: string | IdObj | null): string | null;
export declare function identify(obj: string | IdObj): string;
export declare function mergeRefs(...refs: any): (instance: any) => void;
export declare function safeRun<T extends (...args: any[]) => any>(fn: T | undefined, ...args: Parameters<T>): any;
export declare function waitFor(fn: () => boolean): Promise<void>;
export declare function getInsertIndex(tree: TreeApi<any>): number;
export declare function getInsertParentId(tree: TreeApi<any>): string | null;
