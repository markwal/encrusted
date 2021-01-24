import React from 'react';
import WorkerController from './WorkerController';
import { Graph } from './Rooms';

import Restore from './components/Restore';
import ErrorModal from './components/ErrorModal';


const has = (obj, key) => Object.prototype.hasOwnProperty.call(obj, key);


function debounce(fn, delay = 500) {
  let timer;

  const bounced = (...args) => {
    clearTimeout(timer);

    timer = setTimeout(() => {
      timer = null;
      fn(...args);
    }, delay);
  };

  const cancel = () => clearTimeout(timer);

  return [bounced, cancel];
}


class LocalStore {
  constructor(id) {
    this._id = id;
  }

  set(key, value) {
    const uniqueKey = `${this._id}::${key}`;
    localStorage.setItem(uniqueKey, value);
  }

  get(key) {
    const uniqueKey = `${this._id}::${key}`;
    return localStorage.getItem(uniqueKey);
  }

  has(key) {
    const uniqueKey = `${this._id}::${key}`;
    return has(localStorage, uniqueKey);
  }

  remove(key) {
    const uniqueKey = `${this._id}::${key}`;
    return localStorage.removeItem(uniqueKey);
  }

  clear() {
    const prefix = `${this._id}::`;

    Object.keys(localStorage).forEach((key) => {
      if (key.slice(0, prefix.length) === prefix) {
        localStorage.removeItem(key);
      }
    });
  }
}


const url = process.env.ENCRUSTEDROOT + 'worker.js'

const worker = new WorkerController(url);

let storage; // file specific localstorage
let graph;
let saves = [];
let last_input = '';


const middleware = store => next => (action) => {
  const dispatch = store.dispatch;

  // attach listeners to worker only once
  function bindWorker() {
    // Some actions that are a bit expensive & can be done when idle.
    // * serialize/compress can interrupt rendering, so put it off
    const [saveMap, cancelSave] = debounce(() => {storage.set('map', graph.serialize())});

    worker.on('print', (text) => {
      dispatch({ type: 'TS::TEXT', text });
      cancelSave();
      const countScript = store.getState().transcript.moves.length;
      storage.set(`script::${countScript}`, "<span>&nbsp;" + last_input + "</span><br>" + text);
      storage.set('countScript', countScript);

      if (!!~text.indexOf('You have died')) {
        last_input = 'DIED';
      }
    });

    worker.on('header', data => dispatch({ type: 'TS::HEADER', data }));
    worker.on('quit', () => dispatch({ type: 'TS::QUIT' }));

    // short timer here to make sure the text gets rendered quickest
    worker.on('map', data => setTimeout(() => {
      const [id, name] = JSON.parse(data);

      if (graph && !graph.isCurrent(id)) {
        graph.moveTo(id, name, last_input);
        dispatch({ type: 'MAP::UPDATE' });
        saveMap();
      }
    }, 10));

    // short timer here too
    worker.on('tree', data => setTimeout(() => {
      dispatch({ type: 'TREE::DATA', data });
      storage.set('tree', data);
    }, 10));

    // here too
    worker.on('instructions', data => setTimeout(() => {
      dispatch({ type: 'INSTRUCTIONS', data });
    }, 10));

    // here too, longer delay ok
    worker.on('savestate', (save) => {
      dispatch({ type: 'SAVES::STATE', save });
      // edge case: don't save state at very start of a game
      if (!!last_input) {
        storage.set('savestate', save);
      }
    });

    worker.on('save', (save) => {
      dispatch({ type: 'SAVES::INSTR', save });
      saves.push(save);
      storage.set('saves', JSON.stringify(saves));
    });

    worker.on('restore', () => {
      dispatch({ type: 'MODAL::SHOW', child: <Restore /> });
    });

    worker.on('error', (err) => {
      dispatch({ type: 'MODAL::SHOW', child: <ErrorModal err={err} /> });
    });

    worker.isBound = true;
  }


  // loads a file into the zmachine
  function load(filename, file) {
    // loads into worker
    worker.load(filename, file);

    // set up UI
    last_input = '';
    storage = new LocalStore(filename);
    graph = Graph.deserialize(storage.get('map'));
    saves = JSON.parse(storage.get('saves') || '[]');

    // specific fn to get object details for the tree
    const getDetails = id => worker.sendAnd('getDetails', id);

    dispatch({ type: 'MAP::CREATE', graph });
    dispatch({ type: 'TREE::DATA', data: storage.get('tree') || '{}' });
    dispatch({ type: 'TREE::DETAILS', getDetails });
    dispatch({ type: 'SAVES::LOAD', saves });

    // once its been loaded, check if there is a previous state to restore
    worker.once('loaded', () => {
      const enabled = !!JSON.parse(localStorage.getItem('setting:instructions'));
      worker.send('enable:instructions', !!enabled);

      const query = new URL(window.location.href).searchParams.get('save');
      const state = (query)
        ? decodeURI(query).replace(/ /g, '+')
        : storage.get('savestate');

      let text;

      if (state) {
        const [_id, data] = JSON.parse(state);
        worker.send('load_savestate', data);

        if (!query) {
          // accumulate dead (no undo) persisted transcript
          const countScript = storage.get('countScript');
          if (countScript) {
            text = "";
            for (let i = 0; i <= countScript; i++) {
              let s = storage.get(`script::${i}`);
              if (s) {
                text = text + s;
                storage.remove(`script::${i}`);
              }
            }
            storage.set('script::0', text);
          }
        }
      }

      worker.send('start');
      if (text)
        dispatch({ type: 'TS::TEXT', text });
    });
  }


  switch (action.type) {
    case 'PRELOAD':
      if (!worker.isBound) bindWorker();
      worker.send('instantiate');
      break;

    case 'TS::START':
      if (!worker.isBound) bindWorker();
      load(action.filename, action.file);

      next(action);
      break;

    case 'TS::RESTART':
      worker.send('restart');
      worker.once('loaded', () => worker.send('start'));

      storage.clear();
      last_input = '';
      graph = new Graph();
      dispatch({ type: 'MAP::CREATE', graph });

      next(action);
      break;

    case 'SETTING':
      if (action.name === 'instructions') {
        localStorage.setItem('setting:instructions', action.value);
        worker.send('enable:instructions', action.value);
      }

      next(action);
      break;

    case 'MAP::CLEAR':
      const current = graph.current;
      graph = new Graph();

      if (current) {
        graph.current = current;
        graph.nodes.set(current.id, current);
      }

      dispatch({ type: 'MAP::CREATE', graph });
      break;

    case 'TS::UNDO':
      last_input = 'UNDO';
      worker.send('undo');

      next(action);
      break;

    case 'TS::REDO':
      last_input = 'REDO';
      worker.send('redo');

      next(action);
      break;

    case 'TS::SUBMIT':
      last_input = action.input;
      worker.send('input', action.input);

      next(action);
      break;

    case 'SAVES::RESTORE':
      worker.send('restore', action.data);
      dispatch({ type: 'MODAL::HIDE' });
      break;

    default:
      next(action);
  }
};


export default middleware;
