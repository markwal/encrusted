import React from 'react';
import { createRoot } from 'react-dom/client';
import { createStore, applyMiddleware } from 'redux';
import { Provider } from 'react-redux';
import { MemoryRouter, Routes, Route } from 'react-router-dom';

import ZMachine from './components/ZMachine';
import Launcher from './components/Launcher';

import middleware from './middleware';
import reducer from './reducer';

const store = createStore(reducer, applyMiddleware(middleware));

const container = document.getElementById('root');
const root = createRoot(container);

root.render(
  <Provider store={store}>
    <MemoryRouter initialEntries={['/']} initialIndex={0}>
      <Routes>
        <Route path="/" element={<Launcher />} />
        <Route path="/run/:filename" element={<ZMachine />} />
      </Routes>
    </MemoryRouter>
  </Provider>
);
