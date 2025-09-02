"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.indexOf = exports.isDescendant = void 0;
exports.bound = bound;
exports.isItem = isItem;
exports.isClosed = isClosed;
exports.isOpenWithEmptyChildren = isOpenWithEmptyChildren;
exports.noop = noop;
exports.dfs = dfs;
exports.walk = walk;
exports.focusNextElement = focusNextElement;
exports.focusPrevElement = focusPrevElement;
exports.access = access;
exports.identifyNull = identifyNull;
exports.identify = identify;
exports.mergeRefs = mergeRefs;
exports.safeRun = safeRun;
exports.waitFor = waitFor;
exports.getInsertIndex = getInsertIndex;
exports.getInsertParentId = getInsertParentId;
function bound(n, min, max) {
    return Math.max(Math.min(n, max), min);
}
function isItem(node) {
    return node && node.isLeaf;
}
function isClosed(node) {
    return node && node.isInternal && !node.isOpen;
}
function isOpenWithEmptyChildren(node) {
    var _a;
    return node && node.isOpen && !((_a = node.children) === null || _a === void 0 ? void 0 : _a.length);
}
/**
 * Is first param a descendant of the second param
 */
const isDescendant = (a, b) => {
    let n = a;
    while (n) {
        if (n.id === b.id)
            return true;
        n = n.parent;
    }
    return false;
};
exports.isDescendant = isDescendant;
const indexOf = (node) => {
    if (!node.parent)
        throw Error("Node does not have a parent");
    return node.parent.children.findIndex((c) => c.id === node.id);
};
exports.indexOf = indexOf;
function noop() { }
function dfs(node, id) {
    if (!node)
        return null;
    if (node.id === id)
        return node;
    if (node.children) {
        for (let child of node.children) {
            const result = dfs(child, id);
            if (result)
                return result;
        }
    }
    return null;
}
function walk(node, fn) {
    fn(node);
    if (node.children) {
        for (let child of node.children) {
            walk(child, fn);
        }
    }
}
function focusNextElement(target) {
    const elements = getFocusable(target);
    let next;
    for (let i = 0; i < elements.length; ++i) {
        const item = elements[i];
        if (item === target) {
            next = nextItem(elements, i);
            break;
        }
    }
    // @ts-ignore ??
    next === null || next === void 0 ? void 0 : next.focus();
}
function focusPrevElement(target) {
    const elements = getFocusable(target);
    let next;
    for (let i = 0; i < elements.length; ++i) {
        const item = elements[i];
        if (item === target) {
            next = prevItem(elements, i);
            break;
        }
    }
    // @ts-ignore
    next === null || next === void 0 ? void 0 : next.focus();
}
function nextItem(list, index) {
    if (index + 1 < list.length) {
        return list[index + 1];
    }
    else {
        return list[0];
    }
}
function prevItem(list, index) {
    if (index - 1 >= 0) {
        return list[index - 1];
    }
    else {
        return list[list.length - 1];
    }
}
function getFocusable(target) {
    return Array.from(document.querySelectorAll('button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"]):not([disabled]), details:not([disabled]), summary:not(:disabled)')).filter((e) => e === target || !target.contains(e));
}
function access(obj, accessor) {
    if (typeof accessor === "boolean")
        return accessor;
    if (typeof accessor === "string")
        return obj[accessor];
    return accessor(obj);
}
function identifyNull(obj) {
    if (obj === null)
        return null;
    else
        return identify(obj);
}
function identify(obj) {
    return typeof obj === "string" ? obj : obj.id;
}
function mergeRefs(...refs) {
    return (instance) => {
        refs.forEach((ref) => {
            if (typeof ref === "function") {
                ref(instance);
            }
            else if (ref != null) {
                ref.current = instance;
            }
        });
    };
}
function safeRun(fn, ...args) {
    if (fn)
        return fn(...args);
}
function waitFor(fn) {
    return new Promise((resolve, reject) => {
        let tries = 0;
        function check() {
            tries += 1;
            if (tries === 100)
                reject();
            if (fn())
                resolve();
            else
                setTimeout(check, 10);
        }
        check();
    });
}
function getInsertIndex(tree) {
    var _a, _b;
    const focus = tree.focusedNode;
    if (!focus)
        return (_b = (_a = tree.root.children) === null || _a === void 0 ? void 0 : _a.length) !== null && _b !== void 0 ? _b : 0;
    if (focus.isOpen)
        return 0;
    if (focus.parent)
        return focus.childIndex + 1;
    return 0;
}
function getInsertParentId(tree) {
    const focus = tree.focusedNode;
    if (!focus)
        return null;
    if (focus.isOpen)
        return focus.id;
    if (focus.parent && !focus.parent.isRoot)
        return focus.parent.id;
    return null;
}
