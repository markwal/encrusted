import React from 'react';
import { createRoot } from 'react-dom/client';
import { createStore, applyMiddleware } from 'redux';
import { Provider } from 'react-redux';
import { BrowserRouter, Routes, Route } from 'react-router-dom';

import ZMachine from './components/ZMachine';
import Launcher from './components/Launcher';

import middleware from './middleware';
import reducer from './reducer';

const store = createStore(reducer, applyMiddleware(middleware));

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
