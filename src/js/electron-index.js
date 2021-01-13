import React from 'react';
import ReactDOM from 'react-dom';
import { createStore, applyMiddleware } from 'redux';
import { Provider } from 'react-redux';
import { MemoryRouter, Switch, Route } from 'react-router-dom';

import ZMachine from './components/ZMachine';
import Launcher from './components/Launcher';

import middleware from './middleware';
import reducer from './reducer';

const store = createStore(reducer, applyMiddleware(middleware));

const basename = process.env.ENCRUSTEDROOT;
console.log('basename at init: ', basename)
console.log('window.location.pathname is ', window.location.pathname)

ReactDOM.render(
  <Provider store={store}>
    <MemoryRouter initialEntries={["/"]} initialIndex={1}>
      <Switch>
        <Route exact path="/" component={Launcher} />
        <Route path="/run/:filename" component={ZMachine} />
      </Switch>
    </MemoryRouter>
  </Provider>,
  document.getElementById('root')
);
