const wasmUrl = './aln_autonomous_daemon.wasm';

async function triggerAutonomousRun() {
    const wasm = await WebAssembly.instantiateStreaming(fetch(wasmUrl), {});
    const { start_daemon } = wasm.instance.exports;
    start_daemon(); // immediate autonomous execution
    console.log("[ALN-WASM] Autonomous run triggered — compatibility paradigms active");
}

if (typeof window !== 'undefined') {
    window.addEventListener('load', triggerAutonomousRun);
} else {
    triggerAutonomousRun(); // Node/WebView background
}
