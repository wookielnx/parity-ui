import 'babel-polyfill';

import React from 'react';
import { Provider } from 'react-redux';
import ReactDOM from 'react-dom';

// Needed for onTouchTap
// http://stackoverflow.com/a/34015469/988941
import injectTapEventPlugin from 'react-tap-event-plugin';

import Web3 from 'web3';
import { Web3Provider, MuiThemeProvider, web3Extension } from 'dapps-react-ui';

import 'reset-css/reset.css';
import './index.css';
import './utils/logger';

import Web3WebSocketProvider from './providers/web3WebsocketProvider';
import WsProvider from './providers/wsProvider';

import middlewares from './middlewares';
import createStore from './store/configureStore';
import Routes from './routes';

export default function app (initToken, tokenSetter, addTokenListener, parityPath) {
  const web3WebSocketProvider = new Web3WebSocketProvider(initToken, addTokenListener, parityPath);
  const web3 = new Web3(web3WebSocketProvider);
  web3._extend(web3Extension(web3));

  const store = createStore(middlewares(initToken, tokenSetter, parityPath), parityPath);

  injectTapEventPlugin();
  ReactDOM.render(
    <Provider store={ store }>
      <Web3Provider web3={ web3 }>
        <MuiThemeProvider>
          <Routes store={ store } />
        </MuiThemeProvider>
      </Web3Provider>
    </Provider>,
    document.querySelector('#root')
  );

  const wsProvider = new WsProvider(store, parityPath, addTokenListener);

  wsProvider.init(initToken);
}

// expose globally for parity builtin sysui dapp
global.paritySysuiApp = app;
