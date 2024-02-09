let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}


const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

let cachedFloat64Memory0 = null;

function getFloat64Memory0() {
    if (cachedFloat64Memory0 === null || cachedFloat64Memory0.byteLength === 0) {
        cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
function __wbg_adapter_34(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h21e7125c17c3c772(arg0, arg1);
}

function __wbg_adapter_37(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4f3e3bfcf182d245(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_40(arg0, arg1, arg2, arg3) {
    wasm._dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h652fcf3b1937426c(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

function __wbg_adapter_51(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures__invoke0_mut__h720334607e2c6240(arg0, arg1);
}

function __wbg_adapter_54(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures__invoke1_mut__h1cdaa329018a9dca(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_67(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures__invoke1_mut__h65c2715ed4235977(arg0, arg1, addHeapObject(arg2));
}

/**
* @returns {Promise<void>}
*/
export function run_or_die() {
    wasm.run_or_die();
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}

let cachedFloat32Memory0 = null;

function getFloat32Memory0() {
    if (cachedFloat32Memory0 === null || cachedFloat32Memory0.byteLength === 0) {
        cachedFloat32Memory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32Memory0;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

export function __wbindgen_is_undefined(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
};

export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
};

export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
};

export function __wbindgen_cb_drop(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

export function __wbindgen_string_new(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

export function __wbg_new_abda76e883ba8a5f() {
    const ret = new Error();
    return addHeapObject(ret);
};

export function __wbg_stack_658279fe44541cf6(arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_error_f851667af71bcfc6(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
};

export function __wbindgen_string_get(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbindgen_boolean_get(arg0) {
    const v = getObject(arg0);
    const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
    return ret;
};

export function __wbindgen_number_get(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
    getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
};

export function __wbindgen_number_new(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
};

export function __wbg_crypto_d05b68a3572bb8ca(arg0) {
    const ret = getObject(arg0).crypto;
    return addHeapObject(ret);
};

export function __wbindgen_is_object(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

export function __wbg_process_b02b3570280d0366(arg0) {
    const ret = getObject(arg0).process;
    return addHeapObject(ret);
};

export function __wbg_versions_c1cb42213cedf0f5(arg0) {
    const ret = getObject(arg0).versions;
    return addHeapObject(ret);
};

export function __wbg_node_43b1089f407e4ec2(arg0) {
    const ret = getObject(arg0).node;
    return addHeapObject(ret);
};

export function __wbindgen_is_string(arg0) {
    const ret = typeof(getObject(arg0)) === 'string';
    return ret;
};

export function __wbg_msCrypto_10fc94afee92bd76(arg0) {
    const ret = getObject(arg0).msCrypto;
    return addHeapObject(ret);
};

export function __wbg_require_9a7e0f667ead4995() { return handleError(function () {
    const ret = module.require;
    return addHeapObject(ret);
}, arguments) };

export function __wbindgen_is_function(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
};

export function __wbg_randomFillSync_b70ccbdf4926a99d() { return handleError(function (arg0, arg1) {
    getObject(arg0).randomFillSync(takeObject(arg1));
}, arguments) };

export function __wbg_getRandomValues_7e42b4fb8779dc6d() { return handleError(function (arg0, arg1) {
    getObject(arg0).getRandomValues(getObject(arg1));
}, arguments) };

export function __wbindgen_is_null(arg0) {
    const ret = getObject(arg0) === null;
    return ret;
};

export function __wbg_offsetX_2a15015b9df991ec(arg0) {
    const ret = getObject(arg0).offsetX;
    return ret;
};

export function __wbg_offsetY_f4992c922228f662(arg0) {
    const ret = getObject(arg0).offsetY;
    return ret;
};

export function __wbg_scheduler_181be421fd0b2855(arg0) {
    const ret = getObject(arg0).scheduler;
    return addHeapObject(ret);
};

export function __wbg_scheduler_c3c850461f2d7b5f(arg0) {
    const ret = getObject(arg0).scheduler;
    return addHeapObject(ret);
};

export function __wbg_requestIdleCallback_f8f727e4ca7842d0(arg0) {
    const ret = getObject(arg0).requestIdleCallback;
    return addHeapObject(ret);
};

export function __wbg_postTask_319d3986858dc461(arg0, arg1, arg2) {
    const ret = getObject(arg0).postTask(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};

export function __wbg_prototype_d761fd8c272c3aee() {
    const ret = ResizeObserverEntry.prototype;
    return addHeapObject(ret);
};

export function __wbg_Window_c0d141cc7e9b1f6d(arg0) {
    const ret = getObject(arg0).Window;
    return addHeapObject(ret);
};

export function __wbg_requestFullscreen_5a116c6464189b61(arg0) {
    const ret = getObject(arg0).requestFullscreen;
    return addHeapObject(ret);
};

export function __wbg_onpointerrawupdate_2641d497db2638e4(arg0) {
    const ret = getObject(arg0).onpointerrawupdate;
    return addHeapObject(ret);
};

export function __wbg_getCoalescedEvents_806e5358e1f3130b(arg0) {
    const ret = getObject(arg0).getCoalescedEvents;
    return addHeapObject(ret);
};

export function __wbg_requestFullscreen_7a35806115b07885(arg0) {
    const ret = getObject(arg0).requestFullscreen();
    return addHeapObject(ret);
};

export function __wbg_webkitFullscreenElement_3a363126a251fcd9(arg0) {
    const ret = getObject(arg0).webkitFullscreenElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_webkitRequestFullscreen_db1af6d8d06ed38d(arg0) {
    getObject(arg0).webkitRequestFullscreen();
};

export function __wbg_queueMicrotask_f61ee94ee663068b(arg0) {
    queueMicrotask(getObject(arg0));
};

export function __wbg_queueMicrotask_f82fc5d1e8f816ae(arg0) {
    const ret = getObject(arg0).queueMicrotask;
    return addHeapObject(ret);
};

export function __wbg_instanceof_WebGl2RenderingContext_f921526c513bf717(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof WebGL2RenderingContext;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_beginQuery_d338463adf721553(arg0, arg1, arg2) {
    getObject(arg0).beginQuery(arg1 >>> 0, getObject(arg2));
};

export function __wbg_bindBufferRange_d8a5ebc8ea8be507(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).bindBufferRange(arg1 >>> 0, arg2 >>> 0, getObject(arg3), arg4, arg5);
};

export function __wbg_bindSampler_d74e398b68cf5980(arg0, arg1, arg2) {
    getObject(arg0).bindSampler(arg1 >>> 0, getObject(arg2));
};

export function __wbg_bindVertexArray_8863a216d7b0a339(arg0, arg1) {
    getObject(arg0).bindVertexArray(getObject(arg1));
};

export function __wbg_blitFramebuffer_e6642748dd06d47e(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
    getObject(arg0).blitFramebuffer(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0);
};

export function __wbg_bufferData_496bbb31639d9850(arg0, arg1, arg2, arg3) {
    getObject(arg0).bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
};

export function __wbg_bufferData_21334671c4ba6004(arg0, arg1, arg2, arg3) {
    getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
};

export function __wbg_bufferSubData_c472b93c9e272eac(arg0, arg1, arg2, arg3) {
    getObject(arg0).bufferSubData(arg1 >>> 0, arg2, getObject(arg3));
};

export function __wbg_clearBufferfi_25bcd35b825f629d(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).clearBufferfi(arg1 >>> 0, arg2, arg3, arg4);
};

export function __wbg_clearBufferfv_9de0cb45cc5a012b(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).clearBufferfv(arg1 >>> 0, arg2, getArrayF32FromWasm0(arg3, arg4));
};

export function __wbg_clearBufferiv_fc2f8bce2930c586(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).clearBufferiv(arg1 >>> 0, arg2, getArrayI32FromWasm0(arg3, arg4));
};

export function __wbg_clearBufferuiv_2f6d220a31eabca4(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).clearBufferuiv(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4));
};

export function __wbg_clientWaitSync_6a74725ec890efdd(arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).clientWaitSync(getObject(arg1), arg2 >>> 0, arg3 >>> 0);
    return ret;
};

export function __wbg_compressedTexSubImage2D_945ba54869f3a612(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    getObject(arg0).compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8, arg9);
};

export function __wbg_compressedTexSubImage2D_ed56fa2f82a839b1(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
    getObject(arg0).compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, getObject(arg8));
};

export function __wbg_compressedTexSubImage3D_4cebeae1440fdc14(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    getObject(arg0).compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10, arg11);
};

export function __wbg_compressedTexSubImage3D_0ae61aaa91089745(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
    getObject(arg0).compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, getObject(arg10));
};

export function __wbg_copyBufferSubData_d112912c90270156(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).copyBufferSubData(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
};

export function __wbg_copyTexSubImage3D_9fa5e9e7b16cf09d(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    getObject(arg0).copyTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
};

export function __wbg_createQuery_2ef2dc0f01a4a8e3(arg0) {
    const ret = getObject(arg0).createQuery();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createSampler_039ecd204675292b(arg0) {
    const ret = getObject(arg0).createSampler();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createVertexArray_51d51e1e1e13e9f6(arg0) {
    const ret = getObject(arg0).createVertexArray();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_deleteQuery_0c64c5200cdc57a6(arg0, arg1) {
    getObject(arg0).deleteQuery(getObject(arg1));
};

export function __wbg_deleteSampler_ce5b8e120f96fc1a(arg0, arg1) {
    getObject(arg0).deleteSampler(getObject(arg1));
};

export function __wbg_deleteSync_1b05dfcc176e7466(arg0, arg1) {
    getObject(arg0).deleteSync(getObject(arg1));
};

export function __wbg_deleteVertexArray_3e4f2e2ff7f05a19(arg0, arg1) {
    getObject(arg0).deleteVertexArray(getObject(arg1));
};

export function __wbg_drawArraysInstanced_8fb13fe9faf95212(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).drawArraysInstanced(arg1 >>> 0, arg2, arg3, arg4);
};

export function __wbg_drawBuffers_15d26e17a8d24ee0(arg0, arg1) {
    getObject(arg0).drawBuffers(getObject(arg1));
};

export function __wbg_drawElementsInstanced_dcf53461a977d44c(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawElementsInstanced(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
};

export function __wbg_endQuery_faf7eb231d6f2a66(arg0, arg1) {
    getObject(arg0).endQuery(arg1 >>> 0);
};

export function __wbg_fenceSync_d68dcbdcdd134d92(arg0, arg1, arg2) {
    const ret = getObject(arg0).fenceSync(arg1 >>> 0, arg2 >>> 0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_framebufferTextureLayer_a92788e5f0409234(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).framebufferTextureLayer(arg1 >>> 0, arg2 >>> 0, getObject(arg3), arg4, arg5);
};

export function __wbg_getBufferSubData_8710cc73621fc332(arg0, arg1, arg2, arg3) {
    getObject(arg0).getBufferSubData(arg1 >>> 0, arg2, getObject(arg3));
};

export function __wbg_getIndexedParameter_4f004dc25c3d15a9() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getIndexedParameter(arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getSyncParameter_d93ec7f6bb11274c(arg0, arg1, arg2) {
    const ret = getObject(arg0).getSyncParameter(getObject(arg1), arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_getUniformBlockIndex_99c15053c9a87c73(arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).getUniformBlockIndex(getObject(arg1), getStringFromWasm0(arg2, arg3));
    return ret;
};

export function __wbg_invalidateFramebuffer_03bd99588b15d627() { return handleError(function (arg0, arg1, arg2) {
    getObject(arg0).invalidateFramebuffer(arg1 >>> 0, getObject(arg2));
}, arguments) };

export function __wbg_readBuffer_c426fe18344296ff(arg0, arg1) {
    getObject(arg0).readBuffer(arg1 >>> 0);
};

export function __wbg_readPixels_99fda83f6ca7ec72() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    getObject(arg0).readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, getObject(arg7));
}, arguments) };

export function __wbg_readPixels_9634f0dcfb54667c() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    getObject(arg0).readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
}, arguments) };

export function __wbg_renderbufferStorageMultisample_9260e2e620c949e5(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).renderbufferStorageMultisample(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
};

export function __wbg_samplerParameterf_1c7562ef061b803b(arg0, arg1, arg2, arg3) {
    getObject(arg0).samplerParameterf(getObject(arg1), arg2 >>> 0, arg3);
};

export function __wbg_samplerParameteri_0fee083bc48e70ee(arg0, arg1, arg2, arg3) {
    getObject(arg0).samplerParameteri(getObject(arg1), arg2 >>> 0, arg3);
};

export function __wbg_texStorage2D_6665d01025a7e7fc(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).texStorage2D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
};

export function __wbg_texStorage3D_c01c31c1b02d75fd(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).texStorage3D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5, arg6);
};

export function __wbg_texSubImage2D_d2841ded12a8aa66() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
}, arguments) };

export function __wbg_texSubImage2D_bccf4e250f1ce1b8() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
}, arguments) };

export function __wbg_texSubImage2D_780a7c889dc20a98() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
}, arguments) };

export function __wbg_texSubImage2D_b5bb36f2f54b4264() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
}, arguments) };

export function __wbg_texSubImage2D_a297114050ea1098() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
}, arguments) };

export function __wbg_texSubImage3D_43f39a73ed02fae3() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
}, arguments) };

export function __wbg_texSubImage3D_ffdccca1422b482a() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
}, arguments) };

export function __wbg_texSubImage3D_69d5e09d45e0251c() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
}, arguments) };

export function __wbg_texSubImage3D_ae3ed5d0154c346c() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
}, arguments) };

export function __wbg_texSubImage3D_80693fc2c7855e4d() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
    getObject(arg0).texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, getObject(arg11));
}, arguments) };

export function __wbg_uniform2fv_2b473f6dce24c898(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform2fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
};

export function __wbg_uniform2iv_fdaa3cd258d3451e(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform2iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
};

export function __wbg_uniform3fv_3e55033ca16ec6ab(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform3fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
};

export function __wbg_uniform3iv_3d3ed90c76e6777e(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform3iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
};

export function __wbg_uniform4fv_26ec0c9d7bf6d7c6(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform4fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
};

export function __wbg_uniform4iv_2be6b77c47b90d81(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform4iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
};

export function __wbg_uniformBlockBinding_0dc4bd81bb4ccb6a(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniformBlockBinding(getObject(arg1), arg2 >>> 0, arg3 >>> 0);
};

export function __wbg_uniformMatrix2fv_1ab7aeb8562ea3dd(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).uniformMatrix2fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
};

export function __wbg_uniformMatrix3fv_0b151be4d76ee66b(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).uniformMatrix3fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
};

export function __wbg_uniformMatrix4fv_766b5ba343983038(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).uniformMatrix4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
};

export function __wbg_vertexAttribDivisor_197e2e23e3fbde7f(arg0, arg1, arg2) {
    getObject(arg0).vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
};

export function __wbg_vertexAttribIPointer_6f8540e358f8a547(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).vertexAttribIPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
};

export function __wbg_activeTexture_799bf1387e911c27(arg0, arg1) {
    getObject(arg0).activeTexture(arg1 >>> 0);
};

export function __wbg_attachShader_47256b6b3d42a22e(arg0, arg1, arg2) {
    getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
};

export function __wbg_bindBuffer_24f6010e273fa400(arg0, arg1, arg2) {
    getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
};

export function __wbg_bindFramebuffer_a9573e340dab20fe(arg0, arg1, arg2) {
    getObject(arg0).bindFramebuffer(arg1 >>> 0, getObject(arg2));
};

export function __wbg_bindRenderbuffer_54c404711f9b6958(arg0, arg1, arg2) {
    getObject(arg0).bindRenderbuffer(arg1 >>> 0, getObject(arg2));
};

export function __wbg_bindTexture_92d6d7f8bff9531e(arg0, arg1, arg2) {
    getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
};

export function __wbg_blendColor_7974f09cb60d2be0(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).blendColor(arg1, arg2, arg3, arg4);
};

export function __wbg_blendEquation_12146cb96dc1bcd9(arg0, arg1) {
    getObject(arg0).blendEquation(arg1 >>> 0);
};

export function __wbg_blendEquationSeparate_205526dad772d160(arg0, arg1, arg2) {
    getObject(arg0).blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
};

export function __wbg_blendFunc_533de6de45b80a09(arg0, arg1, arg2) {
    getObject(arg0).blendFunc(arg1 >>> 0, arg2 >>> 0);
};

export function __wbg_blendFuncSeparate_fbf93dee3e5ce456(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export function __wbg_colorMask_fba1e2efd891e2ac(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
};

export function __wbg_compileShader_6bf78b425d5c98e1(arg0, arg1) {
    getObject(arg0).compileShader(getObject(arg1));
};

export function __wbg_copyTexSubImage2D_26685100d5f2b4c0(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
    getObject(arg0).copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
};

export function __wbg_createBuffer_323425af422748ac(arg0) {
    const ret = getObject(arg0).createBuffer();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createFramebuffer_1684a99697ac9563(arg0) {
    const ret = getObject(arg0).createFramebuffer();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createProgram_4eaf3b97b5747a62(arg0) {
    const ret = getObject(arg0).createProgram();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createRenderbuffer_3e6dd356d7897ed7(arg0) {
    const ret = getObject(arg0).createRenderbuffer();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createShader_429776c9dd6fb87b(arg0, arg1) {
    const ret = getObject(arg0).createShader(arg1 >>> 0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createTexture_1bf4d6fec570124b(arg0) {
    const ret = getObject(arg0).createTexture();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_cullFace_6daa9f2aa42b4620(arg0, arg1) {
    getObject(arg0).cullFace(arg1 >>> 0);
};

export function __wbg_deleteBuffer_2c09d03fa4b0bd08(arg0, arg1) {
    getObject(arg0).deleteBuffer(getObject(arg1));
};

export function __wbg_deleteFramebuffer_edd16bb8df6a8e0d(arg0, arg1) {
    getObject(arg0).deleteFramebuffer(getObject(arg1));
};

export function __wbg_deleteProgram_53a32852f245b839(arg0, arg1) {
    getObject(arg0).deleteProgram(getObject(arg1));
};

export function __wbg_deleteRenderbuffer_134040051fcc1ba5(arg0, arg1) {
    getObject(arg0).deleteRenderbuffer(getObject(arg1));
};

export function __wbg_deleteShader_7c1222349324b5e2(arg0, arg1) {
    getObject(arg0).deleteShader(getObject(arg1));
};

export function __wbg_deleteTexture_4fcfea73cd8f6214(arg0, arg1) {
    getObject(arg0).deleteTexture(getObject(arg1));
};

export function __wbg_depthFunc_fb41ad353d07948d(arg0, arg1) {
    getObject(arg0).depthFunc(arg1 >>> 0);
};

export function __wbg_depthMask_6a4ff02cd2a2702e(arg0, arg1) {
    getObject(arg0).depthMask(arg1 !== 0);
};

export function __wbg_depthRange_a5647a9040aec55b(arg0, arg1, arg2) {
    getObject(arg0).depthRange(arg1, arg2);
};

export function __wbg_disable_e02106ca6c7002d6(arg0, arg1) {
    getObject(arg0).disable(arg1 >>> 0);
};

export function __wbg_disableVertexAttribArray_6d57776c8f642f44(arg0, arg1) {
    getObject(arg0).disableVertexAttribArray(arg1 >>> 0);
};

export function __wbg_drawArrays_c91ce3f736bf1f2a(arg0, arg1, arg2, arg3) {
    getObject(arg0).drawArrays(arg1 >>> 0, arg2, arg3);
};

export function __wbg_enable_195891416c520019(arg0, arg1) {
    getObject(arg0).enable(arg1 >>> 0);
};

export function __wbg_enableVertexAttribArray_8804480c2ea0bb72(arg0, arg1) {
    getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
};

export function __wbg_framebufferRenderbuffer_3ec0983918c2b69d(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4));
};

export function __wbg_framebufferTexture2D_e88fcbd7f8523bb8(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5);
};

export function __wbg_frontFace_786a036f1d643925(arg0, arg1) {
    getObject(arg0).frontFace(arg1 >>> 0);
};

export function __wbg_getActiveUniform_78367ddc7339640b(arg0, arg1, arg2) {
    const ret = getObject(arg0).getActiveUniform(getObject(arg1), arg2 >>> 0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_getExtension_77909f6d51d49d4d() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getExtension(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_getParameter_55b36a787dbbfb74() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).getParameter(arg1 >>> 0);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getProgramInfoLog_b81bc53188e286fa(arg0, arg1, arg2) {
    const ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_getProgramParameter_35522a0bfdfaad27(arg0, arg1, arg2) {
    const ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_getShaderInfoLog_968b93e75477d725(arg0, arg1, arg2) {
    const ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_getShaderParameter_ac2727ae4fe7648e(arg0, arg1, arg2) {
    const ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_getSupportedExtensions_fafc31aab913037d(arg0) {
    const ret = getObject(arg0).getSupportedExtensions();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_getUniformLocation_9f6eb60c560a347b(arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_linkProgram_33998194075d71fb(arg0, arg1) {
    getObject(arg0).linkProgram(getObject(arg1));
};

export function __wbg_pixelStorei_f3a24990aa352fc7(arg0, arg1, arg2) {
    getObject(arg0).pixelStorei(arg1 >>> 0, arg2);
};

export function __wbg_polygonOffset_faca8e73770272ff(arg0, arg1, arg2) {
    getObject(arg0).polygonOffset(arg1, arg2);
};

export function __wbg_renderbufferStorage_987d1af7c9faf5dd(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
};

export function __wbg_scissor_e8e41e1c0a9817c8(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).scissor(arg1, arg2, arg3, arg4);
};

export function __wbg_shaderSource_1cb7c64dc7d1a500(arg0, arg1, arg2, arg3) {
    getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
};

export function __wbg_stencilFuncSeparate_8ff94e24a50a3c45(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
};

export function __wbg_stencilMask_641f92999dd3c3de(arg0, arg1) {
    getObject(arg0).stencilMask(arg1 >>> 0);
};

export function __wbg_stencilMaskSeparate_6b2c8ef22fb3b6d1(arg0, arg1, arg2) {
    getObject(arg0).stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
};

export function __wbg_stencilOpSeparate_38925591af8feb44(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export function __wbg_texParameteri_85dad939f62a15aa(arg0, arg1, arg2, arg3) {
    getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
};

export function __wbg_uniform1f_88379f4e2630bc66(arg0, arg1, arg2) {
    getObject(arg0).uniform1f(getObject(arg1), arg2);
};

export function __wbg_uniform1i_d2e61a6a43889648(arg0, arg1, arg2) {
    getObject(arg0).uniform1i(getObject(arg1), arg2);
};

export function __wbg_uniform4f_a9fd337d4b07f595(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).uniform4f(getObject(arg1), arg2, arg3, arg4, arg5);
};

export function __wbg_useProgram_3683cf6f60939dcd(arg0, arg1) {
    getObject(arg0).useProgram(getObject(arg1));
};

export function __wbg_vertexAttribPointer_316ffe2f0458fde7(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
};

export function __wbg_viewport_fad1ce9e18f741c0(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).viewport(arg1, arg2, arg3, arg4);
};

export function __wbg_instanceof_Window_9029196b662bc42a(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_document_f7ace2b956f30a4f(arg0) {
    const ret = getObject(arg0).document;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_navigator_7c9103698acde322(arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
};

export function __wbg_devicePixelRatio_f9de7bddca0eaf20(arg0) {
    const ret = getObject(arg0).devicePixelRatio;
    return ret;
};

export function __wbg_isSecureContext_4890500d265c48bd(arg0) {
    const ret = getObject(arg0).isSecureContext;
    return ret;
};

export function __wbg_cancelIdleCallback_93fba6286778fa5d(arg0, arg1) {
    getObject(arg0).cancelIdleCallback(arg1 >>> 0);
};

export function __wbg_getComputedStyle_09fd3af3edcbbc36() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).getComputedStyle(getObject(arg1));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_matchMedia_12ef69056e32d0b3() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_requestIdleCallback_cd497d1e5bce8ba1() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestIdleCallback(getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_cancelAnimationFrame_9b68e9588c6543bc() { return handleError(function (arg0, arg1) {
    getObject(arg0).cancelAnimationFrame(arg1);
}, arguments) };

export function __wbg_requestAnimationFrame_d082200514b6674d() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_clearTimeout_220be2fa0577b342(arg0, arg1) {
    getObject(arg0).clearTimeout(arg1);
};

export function __wbg_setTimeout_9434bde3b3009c3e() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).setTimeout(getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_setTimeout_eb1a0d116c26d9f6() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
    return ret;
}, arguments) };

export function __wbg_body_674aec4c1c0910cd(arg0) {
    const ret = getObject(arg0).body;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_visibilityState_d18aa202782db311(arg0) {
    const ret = getObject(arg0).visibilityState;
    return addHeapObject(ret);
};

export function __wbg_activeElement_2cf540415b6fcd5c(arg0) {
    const ret = getObject(arg0).activeElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_fullscreenElement_07d5b77ef6c958c1(arg0) {
    const ret = getObject(arg0).fullscreenElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createElement_4891554b28d3388b() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getElementById_cc0e0d931b0d9a28(arg0, arg1, arg2) {
    const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_querySelector_52ded52c20e23921() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).querySelector(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_setAttribute_e7e80b478b7b8b2f() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_setPointerCapture_e7c29336490bba19() { return handleError(function (arg0, arg1) {
    getObject(arg0).setPointerCapture(arg1);
}, arguments) };

export function __wbg_style_3801009b2339aa94(arg0) {
    const ret = getObject(arg0).style;
    return addHeapObject(ret);
};

export function __wbg_focus_dbcbbbb2a04c0e1f() { return handleError(function (arg0) {
    getObject(arg0).focus();
}, arguments) };

export function __wbg_bufferData_92a3e0b745b0d726(arg0, arg1, arg2, arg3) {
    getObject(arg0).bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
};

export function __wbg_bufferData_a11a9f65f31e7256(arg0, arg1, arg2, arg3) {
    getObject(arg0).bufferData(arg1 >>> 0, getObject(arg2), arg3 >>> 0);
};

export function __wbg_bufferSubData_fca6f1c10273be21(arg0, arg1, arg2, arg3) {
    getObject(arg0).bufferSubData(arg1 >>> 0, arg2, getObject(arg3));
};

export function __wbg_compressedTexSubImage2D_21078c6de0a71aad(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
    getObject(arg0).compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, getObject(arg8));
};

export function __wbg_readPixels_91b0d8854de90477() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
    getObject(arg0).readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, getObject(arg7));
}, arguments) };

export function __wbg_texSubImage2D_f1a31f8045b7f831() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
    getObject(arg0).texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, getObject(arg9));
}, arguments) };

export function __wbg_uniform2fv_c928f6ba0085b381(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform2fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
};

export function __wbg_uniform2iv_7e5f8e7c2f4d4d6a(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform2iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
};

export function __wbg_uniform3fv_1aba437b913c1926(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform3fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
};

export function __wbg_uniform3iv_23751fe4dfcdf539(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform3iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
};

export function __wbg_uniform4fv_7c51c2251d851cb2(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform4fv(getObject(arg1), getArrayF32FromWasm0(arg2, arg3));
};

export function __wbg_uniform4iv_27b49984e9c5d90a(arg0, arg1, arg2, arg3) {
    getObject(arg0).uniform4iv(getObject(arg1), getArrayI32FromWasm0(arg2, arg3));
};

export function __wbg_uniformMatrix2fv_f8f3ef807f196bf1(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).uniformMatrix2fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
};

export function __wbg_uniformMatrix3fv_341eec37953e50c5(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).uniformMatrix3fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
};

export function __wbg_uniformMatrix4fv_465ab8de531f4f78(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).uniformMatrix4fv(getObject(arg1), arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
};

export function __wbg_activeTexture_93b4de60af07da9c(arg0, arg1) {
    getObject(arg0).activeTexture(arg1 >>> 0);
};

export function __wbg_attachShader_b65b695055670cb5(arg0, arg1, arg2) {
    getObject(arg0).attachShader(getObject(arg1), getObject(arg2));
};

export function __wbg_bindBuffer_313561e5bc0e533f(arg0, arg1, arg2) {
    getObject(arg0).bindBuffer(arg1 >>> 0, getObject(arg2));
};

export function __wbg_bindFramebuffer_56bf6536a4ced0ec(arg0, arg1, arg2) {
    getObject(arg0).bindFramebuffer(arg1 >>> 0, getObject(arg2));
};

export function __wbg_bindRenderbuffer_559c7c6b6676dddd(arg0, arg1, arg2) {
    getObject(arg0).bindRenderbuffer(arg1 >>> 0, getObject(arg2));
};

export function __wbg_bindTexture_9cb5c770d1ba2cca(arg0, arg1, arg2) {
    getObject(arg0).bindTexture(arg1 >>> 0, getObject(arg2));
};

export function __wbg_blendColor_b9006ef6c450acd0(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).blendColor(arg1, arg2, arg3, arg4);
};

export function __wbg_blendEquation_f31ce08020786a09(arg0, arg1) {
    getObject(arg0).blendEquation(arg1 >>> 0);
};

export function __wbg_blendEquationSeparate_7ec5e34f066e44f8(arg0, arg1, arg2) {
    getObject(arg0).blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
};

export function __wbg_blendFunc_fbe9d3a688fe71c3(arg0, arg1, arg2) {
    getObject(arg0).blendFunc(arg1 >>> 0, arg2 >>> 0);
};

export function __wbg_blendFuncSeparate_7547ade0a7dfade2(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export function __wbg_colorMask_7cbd7a102954ede9(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
};

export function __wbg_compileShader_d88d0a8cd9b72b4d(arg0, arg1) {
    getObject(arg0).compileShader(getObject(arg1));
};

export function __wbg_copyTexSubImage2D_3029f8dfe7543ab6(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
    getObject(arg0).copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
};

export function __wbg_createBuffer_59051f4461e7c5e2(arg0) {
    const ret = getObject(arg0).createBuffer();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createFramebuffer_223c1212ad76affc(arg0) {
    const ret = getObject(arg0).createFramebuffer();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createProgram_88dbe21c0b682e1a(arg0) {
    const ret = getObject(arg0).createProgram();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createRenderbuffer_bcb61b756ba21490(arg0) {
    const ret = getObject(arg0).createRenderbuffer();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createShader_9d7d388633caad18(arg0, arg1) {
    const ret = getObject(arg0).createShader(arg1 >>> 0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createTexture_9d0bb4d741b8ad76(arg0) {
    const ret = getObject(arg0).createTexture();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_cullFace_4c086dc1d86a19b5(arg0, arg1) {
    getObject(arg0).cullFace(arg1 >>> 0);
};

export function __wbg_deleteBuffer_cdc6b9c73f54aff7(arg0, arg1) {
    getObject(arg0).deleteBuffer(getObject(arg1));
};

export function __wbg_deleteFramebuffer_fcc10cb143c6573d(arg0, arg1) {
    getObject(arg0).deleteFramebuffer(getObject(arg1));
};

export function __wbg_deleteProgram_d8d7fc79ba83b256(arg0, arg1) {
    getObject(arg0).deleteProgram(getObject(arg1));
};

export function __wbg_deleteRenderbuffer_edf9e1b4e0a1e005(arg0, arg1) {
    getObject(arg0).deleteRenderbuffer(getObject(arg1));
};

export function __wbg_deleteShader_9a2f85efe5cb3706(arg0, arg1) {
    getObject(arg0).deleteShader(getObject(arg1));
};

export function __wbg_deleteTexture_a883356c5034d482(arg0, arg1) {
    getObject(arg0).deleteTexture(getObject(arg1));
};

export function __wbg_depthFunc_4eda7b4e682acbad(arg0, arg1) {
    getObject(arg0).depthFunc(arg1 >>> 0);
};

export function __wbg_depthMask_a3071e13bb087102(arg0, arg1) {
    getObject(arg0).depthMask(arg1 !== 0);
};

export function __wbg_depthRange_ff5298a73fd02650(arg0, arg1, arg2) {
    getObject(arg0).depthRange(arg1, arg2);
};

export function __wbg_disable_5cf2070641fa2ed7(arg0, arg1) {
    getObject(arg0).disable(arg1 >>> 0);
};

export function __wbg_disableVertexAttribArray_8dacd44e21adcaa2(arg0, arg1) {
    getObject(arg0).disableVertexAttribArray(arg1 >>> 0);
};

export function __wbg_drawArrays_d5c7dc2b2376c85a(arg0, arg1, arg2, arg3) {
    getObject(arg0).drawArrays(arg1 >>> 0, arg2, arg3);
};

export function __wbg_enable_8965e69c596f0a94(arg0, arg1) {
    getObject(arg0).enable(arg1 >>> 0);
};

export function __wbg_enableVertexAttribArray_2b0475db43533cf2(arg0, arg1) {
    getObject(arg0).enableVertexAttribArray(arg1 >>> 0);
};

export function __wbg_framebufferRenderbuffer_d80f5381d429bc45(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4));
};

export function __wbg_framebufferTexture2D_953e69a8bec22fa9(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5);
};

export function __wbg_frontFace_0ba67b9e6365557c(arg0, arg1) {
    getObject(arg0).frontFace(arg1 >>> 0);
};

export function __wbg_getActiveUniform_87df972e841afed2(arg0, arg1, arg2) {
    const ret = getObject(arg0).getActiveUniform(getObject(arg1), arg2 >>> 0);
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_getParameter_bfab7f0b00c9d7fb() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).getParameter(arg1 >>> 0);
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getProgramInfoLog_0b7af4ad85fa52a4(arg0, arg1, arg2) {
    const ret = getObject(arg1).getProgramInfoLog(getObject(arg2));
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_getProgramParameter_2a3735278367f8bc(arg0, arg1, arg2) {
    const ret = getObject(arg0).getProgramParameter(getObject(arg1), arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_getShaderInfoLog_979aafa403ffb252(arg0, arg1, arg2) {
    const ret = getObject(arg1).getShaderInfoLog(getObject(arg2));
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_getShaderParameter_e8054f1d9026fb70(arg0, arg1, arg2) {
    const ret = getObject(arg0).getShaderParameter(getObject(arg1), arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_getUniformLocation_688976233799a45a(arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).getUniformLocation(getObject(arg1), getStringFromWasm0(arg2, arg3));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_linkProgram_9a2d12d120d99917(arg0, arg1) {
    getObject(arg0).linkProgram(getObject(arg1));
};

export function __wbg_pixelStorei_5ec932ebefd00149(arg0, arg1, arg2) {
    getObject(arg0).pixelStorei(arg1 >>> 0, arg2);
};

export function __wbg_polygonOffset_55eea57bba1b49e9(arg0, arg1, arg2) {
    getObject(arg0).polygonOffset(arg1, arg2);
};

export function __wbg_renderbufferStorage_4bcd9ddf1749ce26(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
};

export function __wbg_scissor_c8ec3b1e053f3756(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).scissor(arg1, arg2, arg3, arg4);
};

export function __wbg_shaderSource_f435f9b74440bb54(arg0, arg1, arg2, arg3) {
    getObject(arg0).shaderSource(getObject(arg1), getStringFromWasm0(arg2, arg3));
};

export function __wbg_stencilFuncSeparate_0fae0ee7c04a23b2(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
};

export function __wbg_stencilMask_79416c29ac1ce3a4(arg0, arg1) {
    getObject(arg0).stencilMask(arg1 >>> 0);
};

export function __wbg_stencilMaskSeparate_19bdb57664d2c34f(arg0, arg1, arg2) {
    getObject(arg0).stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
};

export function __wbg_stencilOpSeparate_18e0bd316555925f(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export function __wbg_texParameteri_1f17358e51eb8069(arg0, arg1, arg2, arg3) {
    getObject(arg0).texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
};

export function __wbg_uniform1f_7586c5e17ad254c9(arg0, arg1, arg2) {
    getObject(arg0).uniform1f(getObject(arg1), arg2);
};

export function __wbg_uniform1i_9f94ef0ba6b3cc66(arg0, arg1, arg2) {
    getObject(arg0).uniform1i(getObject(arg1), arg2);
};

export function __wbg_uniform4f_9aa5afa9177c6ab1(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).uniform4f(getObject(arg1), arg2, arg3, arg4, arg5);
};

export function __wbg_useProgram_019eb6df066fabf5(arg0, arg1) {
    getObject(arg0).useProgram(getObject(arg1));
};

export function __wbg_vertexAttribPointer_ca11984ee8843c0a(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
};

export function __wbg_viewport_6ebef187c89e2616(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).viewport(arg1, arg2, arg3, arg4);
};

export function __wbg_media_1712dab8d575e095(arg0, arg1) {
    const ret = getObject(arg1).media;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_matches_07c564b5b4101cf2(arg0) {
    const ret = getObject(arg0).matches;
    return ret;
};

export function __wbg_addListener_85fb6e4bd17e8878() { return handleError(function (arg0, arg1) {
    getObject(arg0).addListener(getObject(arg1));
}, arguments) };

export function __wbg_removeListener_3b62020874cfc3c7() { return handleError(function (arg0, arg1) {
    getObject(arg0).removeListener(getObject(arg1));
}, arguments) };

export function __wbg_close_bf02121cc8bcdd76(arg0) {
    getObject(arg0).close();
};

export function __wbg_postMessage_0c659be53ba68fd2() { return handleError(function (arg0, arg1) {
    getObject(arg0).postMessage(getObject(arg1));
}, arguments) };

export function __wbg_start_7090d798fb15dbdb(arg0) {
    getObject(arg0).start();
};

export function __wbg_deltaX_84508d00a1050e70(arg0) {
    const ret = getObject(arg0).deltaX;
    return ret;
};

export function __wbg_deltaY_64823169afb0335d(arg0) {
    const ret = getObject(arg0).deltaY;
    return ret;
};

export function __wbg_deltaMode_1c680147cfdba8a5(arg0) {
    const ret = getObject(arg0).deltaMode;
    return ret;
};

export function __wbg_debug_9a6b3243fbbebb61(arg0) {
    console.debug(getObject(arg0));
};

export function __wbg_error_788ae33f81d3b84b(arg0) {
    console.error(getObject(arg0));
};

export function __wbg_error_c9309504864e78b5(arg0, arg1) {
    console.error(getObject(arg0), getObject(arg1));
};

export function __wbg_info_2e30e8204b29d91d(arg0) {
    console.info(getObject(arg0));
};

export function __wbg_log_1d3ae0273d8f4f8a(arg0) {
    console.log(getObject(arg0));
};

export function __wbg_warn_d60e832f9882c1b2(arg0) {
    console.warn(getObject(arg0));
};

export function __wbg_drawArraysInstancedANGLE_01b862ba133350a3(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).drawArraysInstancedANGLE(arg1 >>> 0, arg2, arg3, arg4);
};

export function __wbg_drawElementsInstancedANGLE_ea6343af8bf9c9f8(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawElementsInstancedANGLE(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
};

export function __wbg_vertexAttribDivisorANGLE_a8476eb778e16c70(arg0, arg1, arg2) {
    getObject(arg0).vertexAttribDivisorANGLE(arg1 >>> 0, arg2 >>> 0);
};

export function __wbg_port1_11e74cca69c6d1d1(arg0) {
    const ret = getObject(arg0).port1;
    return addHeapObject(ret);
};

export function __wbg_port2_084b643abbbceaa7(arg0) {
    const ret = getObject(arg0).port2;
    return addHeapObject(ret);
};

export function __wbg_new_fc733ab8af293900() { return handleError(function () {
    const ret = new MessageChannel();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_appendChild_51339d4cde00ee22() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).appendChild(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_contains_fab0394bee511df3(arg0, arg1) {
    const ret = getObject(arg0).contains(getObject(arg1));
    return ret;
};

export function __wbg_altKey_612289acf855835c(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};

export function __wbg_ctrlKey_582686fb2263dd3c(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};

export function __wbg_shiftKey_48e8701355d8e2d4(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};

export function __wbg_metaKey_43193b7cc99f8914(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};

export function __wbg_location_e5f8d98ba89b596e(arg0) {
    const ret = getObject(arg0).location;
    return ret;
};

export function __wbg_repeat_52850ed66db69aba(arg0) {
    const ret = getObject(arg0).repeat;
    return ret;
};

export function __wbg_key_8aeaa079126a9cc7(arg0, arg1) {
    const ret = getObject(arg1).key;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_code_96d6322b968b2d17(arg0, arg1) {
    const ret = getObject(arg1).code;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_framebufferTextureMultiviewOVR_4d911c3fccedc517(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).framebufferTextureMultiviewOVR(arg1 >>> 0, arg2 >>> 0, getObject(arg3), arg4, arg5, arg6);
};

export function __wbg_persisted_a24f61318c91570a(arg0) {
    const ret = getObject(arg0).persisted;
    return ret;
};

export function __wbg_pointerId_701aab7b4fb073ff(arg0) {
    const ret = getObject(arg0).pointerId;
    return ret;
};

export function __wbg_pressure_e388b6fd623a3917(arg0) {
    const ret = getObject(arg0).pressure;
    return ret;
};

export function __wbg_pointerType_0009b1e4e6b0f428(arg0, arg1) {
    const ret = getObject(arg1).pointerType;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_getCoalescedEvents_927ed71312cc0362(arg0) {
    const ret = getObject(arg0).getCoalescedEvents();
    return addHeapObject(ret);
};

export function __wbg_getPropertyValue_45480a940d0053a0() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_removeProperty_298c1bf4d39723f9() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_setProperty_b95ef63ab852879e() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_preventDefault_24104f3f0a54546a(arg0) {
    getObject(arg0).preventDefault();
};

export function __wbg_contentRect_3a38b9ecf8994843(arg0) {
    const ret = getObject(arg0).contentRect;
    return addHeapObject(ret);
};

export function __wbg_devicePixelContentBoxSize_5b7f524383aabe72(arg0) {
    const ret = getObject(arg0).devicePixelContentBoxSize;
    return addHeapObject(ret);
};

export function __wbg_getSupportedProfiles_420ce044cc3114a1(arg0) {
    const ret = getObject(arg0).getSupportedProfiles();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_instanceof_DomException_11f350c47999e343(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof DOMException;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_message_ad3cc15a4d40c34b(arg0, arg1) {
    const ret = getObject(arg1).message;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_id_2744422b8df66a4f(arg0, arg1) {
    const ret = getObject(arg1).id;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_index_3fa2bac9f2910f3e(arg0) {
    const ret = getObject(arg0).index;
    return ret;
};

export function __wbg_mapping_4620edf85483600a(arg0) {
    const ret = getObject(arg0).mapping;
    return addHeapObject(ret);
};

export function __wbg_connected_7a42d2777c81c280(arg0) {
    const ret = getObject(arg0).connected;
    return ret;
};

export function __wbg_buttons_a308922181d88094(arg0) {
    const ret = getObject(arg0).buttons;
    return addHeapObject(ret);
};

export function __wbg_axes_1b9e57ee3e48f6df(arg0) {
    const ret = getObject(arg0).axes;
    return addHeapObject(ret);
};

export function __wbg_pressed_d881ce4add9c474b(arg0) {
    const ret = getObject(arg0).pressed;
    return ret;
};

export function __wbg_value_c99f8b78cbe90703(arg0) {
    const ret = getObject(arg0).value;
    return ret;
};

export function __wbg_instanceof_HtmlCanvasElement_da5f9efa0688cf6d(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLCanvasElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_width_2931aaedd21f1fff(arg0) {
    const ret = getObject(arg0).width;
    return ret;
};

export function __wbg_setwidth_a667a942dba6656e(arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
};

export function __wbg_height_0d36fbbeb60b0661(arg0) {
    const ret = getObject(arg0).height;
    return ret;
};

export function __wbg_setheight_a747d440760fe5aa(arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
};

export function __wbg_getContext_6d1f155bb5c1096a() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2), getObject(arg3));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_new_6c08de71fd7da8d7() { return handleError(function (arg0) {
    const ret = new IntersectionObserver(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_disconnect_00c020d40f47fb32(arg0) {
    getObject(arg0).disconnect();
};

export function __wbg_observe_0ce70bae387f9bdf(arg0, arg1) {
    getObject(arg0).observe(getObject(arg1));
};

export function __wbg_width_0b4991f5cf7c640a(arg0) {
    const ret = getObject(arg0).width;
    return ret;
};

export function __wbg_setwidth_15266a5e81f43cf0(arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
};

export function __wbg_height_8cce73e95fb10fd3(arg0) {
    const ret = getObject(arg0).height;
    return ret;
};

export function __wbg_setheight_2e9bab573f1775a6(arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
};

export function __wbg_new_9d3e795dcd24a5d9() { return handleError(function (arg0) {
    const ret = new ResizeObserver(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_disconnect_4f00a95c32fad6fe(arg0) {
    getObject(arg0).disconnect();
};

export function __wbg_observe_e3e06d8e2ad2b89c(arg0, arg1) {
    getObject(arg0).observe(getObject(arg1));
};

export function __wbg_observe_5f98196bfaa6b83c(arg0, arg1, arg2) {
    getObject(arg0).observe(getObject(arg1), getObject(arg2));
};

export function __wbg_unobserve_e7fcbda12d671fd7(arg0, arg1) {
    getObject(arg0).unobserve(getObject(arg1));
};

export function __wbg_signal_4bd18fb489af2d4c(arg0) {
    const ret = getObject(arg0).signal;
    return addHeapObject(ret);
};

export function __wbg_new_55c9955722952374() { return handleError(function () {
    const ret = new AbortController();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_abort_654b796176d117aa(arg0) {
    getObject(arg0).abort();
};

export function __wbg_addEventListener_5651108fc3ffeb6e() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_removeEventListener_5de660c02ed784e4() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_width_019b79813c2e80cf(arg0) {
    const ret = getObject(arg0).width;
    return ret;
};

export function __wbg_height_12082005add04bb5(arg0) {
    const ret = getObject(arg0).height;
    return ret;
};

export function __wbg_getGamepads_cbb82980ae1db5e1() { return handleError(function (arg0) {
    const ret = getObject(arg0).getGamepads();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_bindVertexArrayOES_b7d9da7e073aa6b5(arg0, arg1) {
    getObject(arg0).bindVertexArrayOES(getObject(arg1));
};

export function __wbg_createVertexArrayOES_6a3c3a5a68201f8f(arg0) {
    const ret = getObject(arg0).createVertexArrayOES();
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_deleteVertexArrayOES_7bf4589e63d84df6(arg0, arg1) {
    getObject(arg0).deleteVertexArrayOES(getObject(arg1));
};

export function __wbg_width_5728ff70a76f27ac(arg0) {
    const ret = getObject(arg0).width;
    return ret;
};

export function __wbg_height_7fd80bb9bbf69d8c(arg0) {
    const ret = getObject(arg0).height;
    return ret;
};

export function __wbg_isIntersecting_6285c8df39df79f3(arg0) {
    const ret = getObject(arg0).isIntersecting;
    return ret;
};

export function __wbg_ctrlKey_0a805df688b5bf42(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};

export function __wbg_shiftKey_8a070ab6169b5fa4(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};

export function __wbg_altKey_6fc1761a6b7a406e(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};

export function __wbg_metaKey_d89287be4389a3c1(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};

export function __wbg_button_7a095234b69de930(arg0) {
    const ret = getObject(arg0).button;
    return ret;
};

export function __wbg_buttons_d0f40e1650e3fa28(arg0) {
    const ret = getObject(arg0).buttons;
    return ret;
};

export function __wbg_movementX_966ec323c169d1a6(arg0) {
    const ret = getObject(arg0).movementX;
    return ret;
};

export function __wbg_movementY_b14b3bc8e1b31f23(arg0) {
    const ret = getObject(arg0).movementY;
    return ret;
};

export function __wbg_inlineSize_9394a3057e5da0ae(arg0) {
    const ret = getObject(arg0).inlineSize;
    return ret;
};

export function __wbg_blockSize_389e379c0494897e(arg0) {
    const ret = getObject(arg0).blockSize;
    return ret;
};

export function __wbg_size_6eb4aa794f6bf220(arg0) {
    const ret = getObject(arg0).size;
    return ret;
};

export function __wbg_type_37bb6b4936b5e027(arg0) {
    const ret = getObject(arg0).type;
    return ret;
};

export function __wbg_name_ebae3a7e89367611(arg0, arg1) {
    const ret = getObject(arg1).name;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_videoWidth_02eadb74917aa4fc(arg0) {
    const ret = getObject(arg0).videoWidth;
    return ret;
};

export function __wbg_videoHeight_dac4c345988e5562(arg0) {
    const ret = getObject(arg0).videoHeight;
    return ret;
};

export function __wbg_drawBuffersWEBGL_139bf2fe0c1522d6(arg0, arg1) {
    getObject(arg0).drawBuffersWEBGL(getObject(arg1));
};

export function __wbg_performance_eeefc685c9bc38b4(arg0) {
    const ret = getObject(arg0).performance;
    return addHeapObject(ret);
};

export function __wbg_now_e0d8ec93dd25766a(arg0) {
    const ret = getObject(arg0).now();
    return ret;
};

export function __wbg_get_0ee8ea3c7c984c45(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
};

export function __wbg_length_161c0d89c6535c1d(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};

export function __wbg_new_75208e29bddfd88c() {
    const ret = new Array();
    return addHeapObject(ret);
};

export function __wbg_newnoargs_cfecb3965268594c(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbg_call_3f093dd26d5569f8() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_632630b5cec17f21() {
    const ret = new Object();
    return addHeapObject(ret);
};

export function __wbg_self_05040bd9523805b9() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_window_adc720039f2cb14f() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_globalThis_622105db80c1457d() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_global_f56b013ed9bcf359() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_includes_a22cdfddb915d243(arg0, arg1, arg2) {
    const ret = getObject(arg0).includes(getObject(arg1), arg2);
    return ret;
};

export function __wbg_of_14704bfb1b7639cf(arg0) {
    const ret = Array.of(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_push_0239ee92f127e807(arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

export function __wbg_call_67f2111acd2dfdb6() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_now_ba25f0a487340763() {
    const ret = Date.now();
    return ret;
};

export function __wbg_getOwnPropertyDescriptor_16ff6dea996a6c0b(arg0, arg1) {
    const ret = Object.getOwnPropertyDescriptor(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_is_bd5dc4ae269cba1c(arg0, arg1) {
    const ret = Object.is(getObject(arg0), getObject(arg1));
    return ret;
};

export function __wbg_resolve_5da6faf2c96fd1d5(arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_catch_cd2ed55c8688225d(arg0, arg1) {
    const ret = getObject(arg0).catch(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_then_f9e58f5a50f43eae(arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_buffer_b914fb8b50ebbc3e(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_42904a72cefa1e00(arg0, arg1, arg2) {
    const ret = new Int8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_0aafe9b39ed85f71(arg0, arg1, arg2) {
    const ret = new Int16Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_9ca2c1faeb430732(arg0, arg1, arg2) {
    const ret = new Int32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_0de9ee56e9f6ee6e(arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_new_b1f2d6842d615181(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_set_7d988c98e6ced92d(arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
};

export function __wbg_newwithbyteoffsetandlength_8c2171d5a9b7f791(arg0, arg1, arg2) {
    const ret = new Uint16Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_7f2d9491ea8c746e(arg0, arg1, arg2) {
    const ret = new Uint32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_5fd0a60d38f47fa6(arg0, arg1, arg2) {
    const ret = new Float32Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_newwithlength_0d03cef43b68a530(arg0) {
    const ret = new Uint8Array(arg0 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_subarray_adc418253d76e2f1(arg0, arg1, arg2) {
    const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_set_961700853a212a39() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
}, arguments) };

export function __wbindgen_debug_string(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbindgen_throw(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export function __wbindgen_memory() {
    const ret = wasm.memory;
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper540(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 100, __wbg_adapter_34);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper541(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 100, __wbg_adapter_37);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper542(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 100, __wbg_adapter_40);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper543(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 100, __wbg_adapter_37);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper548(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 100, __wbg_adapter_37);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper550(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 100, __wbg_adapter_37);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper552(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 100, __wbg_adapter_37);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper5503(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 2162, __wbg_adapter_51);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper5504(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 2162, __wbg_adapter_54);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper5505(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 2162, __wbg_adapter_54);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper5506(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 2162, __wbg_adapter_54);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper5507(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 2162, __wbg_adapter_54);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper5508(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 2162, __wbg_adapter_54);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper5509(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 2162, __wbg_adapter_54);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper5669(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 2212, __wbg_adapter_67);
    return addHeapObject(ret);
};

