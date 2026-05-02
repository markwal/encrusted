import React from 'react';
import { createRoot } from 'react-dom/client';
import { createStore, applyMiddleware, compose } from 'redux';
import { Provider } from 'react-redux';
import { BrowserRouter, Routes, Route } from 'react-router-dom';

import ZMachine from './components/ZMachine';
import Launcher from './components/Launcher';

import middleware from './middleware';
import reducer from './reducer';

const composeEnhancers = window.__REDUX_DEVTOOLS_EXTENSION_COMPOSE__ || compose;
const store = createStore(reducer, composeEnhancers(applyMiddleware(middleware)));

const basename = process.env.ENCRUSTEDROOT;
const container = document.getElementById('root');
const root = createRoot(container);

root.render(
  <Provider store={store}>
    <BrowserRouter basename={basename}>
      <Routes>
        <Route path="/" element={<Launcher />} />
        <Route path="/run/:filename" element={<ZMachine />} />
      </Routes>
    </BrowserRouter>
  </Provider>
);
