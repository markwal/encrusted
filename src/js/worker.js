import * as wasm from '../../pkg/web_bg.wasm';
import { __wbg_set_wasm, Engine } from '../../pkg/web_bg.js';

// hold onto active file in case of restarts
let file = null;
let engine = null;
let initPromise = null;


function sendWorkerMessage(type, msg) {
  postMessage({ type, msg });
}

globalThis.__encrusted_js_message = sendWorkerMessage;
globalThis.__encrusted_rand = () => Math.floor(Math.random() * 0xFFFF);
globalThis.__encrusted_js_error = (msg) => {
  const err = new Error(msg);

  // some browser devtools try to clean up stacktraces and ruin Rust symbols
  // in the process so we just emit it as a string as part of the message
  // plus we add some as well as some extra whitepace to thwart Safari
  // heuristics to avoid having it mangle the message
  console.error(msg + "\n\nStack:\n\n" + err.stack + "\n\n");
};


function getEngine() {
  if (!engine) {
    throw new Error('Wasm engine has not been loaded yet');
  }

  return engine;
}


function step() {
  const done = getEngine().step();
  if (done) sendWorkerMessage('quit');
}


function instantiate() {
  if (initPromise) {
    return initPromise;
  }

  initPromise = Promise.resolve()
    .then(() => {
      __wbg_set_wasm(wasm);
      wasm.__wbindgen_start();
    })
    .catch((err) => {
      initPromise = null;
      throw err;
    });

  return initPromise;
}


function loadEngine() {
  engine = new Engine(file);
  sendWorkerMessage('loaded');
}


// dispatch handlers based on incoming messages
onmessage = async (ev) => {
  try {
    if (ev.data.type === 'instantiate') {
      await instantiate();
    }

    if (ev.data.type === 'load') {
      await instantiate();
      file = new Uint8Array(ev.data.msg.file);
      loadEngine();
    }

    if (ev.data.type === 'start') {
      step();
    }

    if (ev.data.type === 'restart') {
      loadEngine();
    }

    if (ev.data.type === 'input') {
      getEngine().feed(ev.data.msg);
      step();
    }

    if (ev.data.type === 'restore') {
      getEngine().restore(ev.data.msg);
      step();
    }

    if (ev.data.type === 'load_savestate') {
      getEngine().load_savestate(ev.data.msg);
      step();
    }

    if (ev.data.type === 'undo') {
      const ok = getEngine().undo();

      sendWorkerMessage('undo', ok);
      getEngine().get_updates();
    }

    if (ev.data.type === 'redo') {
      const ok = getEngine().redo();

      sendWorkerMessage('redo', ok);
      getEngine().get_updates();
    }

    if (ev.data.type === 'interpreter_header') {
      console.log("set_terp_caps: ", JSON.stringify(ev.data.msg));
      getEngine().set_terp_caps(JSON.stringify(ev.data.msg));
    }

    if (ev.data.type === 'enable:instructions') {
      getEngine().enable_instruction_logs(!!ev.data.msg);
    }

    if (ev.data.type === 'getDetails') {
      const details = getEngine().get_object_details(ev.data.msg);
      sendWorkerMessage('getDetails', details);
    }
  } catch (err) {
    setTimeout(() => {
      console.log('Error starting wasm: ', err, err.stack);
    });
  }
};
