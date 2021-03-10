import { Wrapper, rust } from 'wasm-ffi';

// const wasmURL = 'web.wasm';
const wasmURL = process.env.ENCRUSTEDROOT + "web.wasm"

// hold onto active file in case of restarts
let file = null;


function sendWorkerMessage(type, msg) {
  postMessage({ type, msg });
}


const zmachine = new Wrapper({
  hook: [],
  create: [null, ['number', 'number']],
  feed: [null, ['string']],
  step: ['bool'],
  undo: ['bool'],
  redo: ['bool'],
  get_updates: [],
  restore: [null, ['string']],
  load_savestate: [null, ['string']],
  enable_instruction_logs: [null, ['bool']],
  get_object_details: [rust.string, ['number']],
  flush_log: [],
  set_terp_caps: [null, ['string']],
});


zmachine.imports(wrap => ({
  env: {
    js_message: wrap('string', 'string', sendWorkerMessage),

    trace: wrap('string', (msg) => {
      const err = new Error(msg);

      setTimeout(() => {
        zmachine.flush_log();
      }, 200);

      postMessage({
        type: 'error',
        msg: { msg, stack: err.stack }
      });
    }),

    rand: function() {
      return Math.floor(Math.random() * 0xFFFF);
    },

    consolelog: wrap('string', (msg) => {
      console.log(msg);
    }),

    js_error: wrap('string', (msg) => {
      var err = new Error();

      // some browser devtools try to clean up stacktraces and ruin Rust symbols
      // in the process so we just emit it as a string as part of the message
      // plus we add some as well as some extra whitepace to thwart Safari
      // heuristics to avoid having it mangle the message
      console.error(msg + "\n\nStack:\n\n" + err.stack + "\n\n");
    })
  },
}));


function step() {
  const done = zmachine.step();
  if (done) sendWorkerMessage('quit');
}


function instantiate() {
  if (zmachine.exports) return Promise.resolve();
  return zmachine.fetch(wasmURL).then(() => zmachine.hook());
}


// dispatch handlers based on incoming messages
onmessage = (ev) => {
  // only want to compile/load the module once
  if (ev.data.type === 'instantiate') {
    instantiate().catch(err => setTimeout(() => {
      console.log('Error starting wasm: ', err, err.stack);
    }));
  }

  if (ev.data.type === 'load') {
    instantiate()
      .then(() => {
        file = new Uint8Array(ev.data.msg.file);
        const file_ptr = zmachine.utils.writeArray(file);

        zmachine.create(file_ptr, file.length);
        sendWorkerMessage('loaded');
      })
      .catch(err => setTimeout(() => {
        console.log('Error starting wasm: ', err, err.stack);
      }));
  }

  if (ev.data.type === 'start') {
    step();
  }

  if (ev.data.type === 'restart') {
    const file_ptr = zmachine.utils.writeArray(file);

    zmachine.create(file_ptr, file.length);
    sendWorkerMessage('interpreter_header', store.getState().interpreter);
    sendWorkerMessage('loaded');
  }

  if (ev.data.type === 'input') {
    zmachine.feed(ev.data.msg);
    step();
  }

  if (ev.data.type === 'restore') {
    zmachine.restore(ev.data.msg);
    step();
  }

  if (ev.data.type === 'load_savestate') {
    zmachine.load_savestate(ev.data.msg);
    step();
  }

  if (ev.data.type === 'undo') {
    const ok = zmachine.undo();

    sendWorkerMessage('undo', ok);
    zmachine.get_updates();
  }

  if (ev.data.type === 'redo') {
    const ok = zmachine.redo();

    sendWorkerMessage('redo', ok);
    zmachine.get_updates();
  }

  if (ev.data.type === 'interpreter_header') {
    console.log("set_terp_caps: ", JSON.stringify(ev.data.msg));
    zmachine.set_terp_caps(JSON.stringify(ev.data.msg));
  }

  if (ev.data.type === 'enable:instructions') {
    zmachine.enable_instruction_logs(!!ev.data.msg);
  }

  if (ev.data.type === 'getDetails') {
    const str = zmachine.get_object_details(ev.data.msg);
    sendWorkerMessage('getDetails', str.value);
    str.free();
  }
};
