import { createStore, applyMiddleware } from 'redux';

import rootReducer from '../reducers';

export default function configure (middlewares, parityPath) {
  const create = window.devToolsExtension
    ? window.devToolsExtension()(createStore)
    : createStore;

  const createStoreWithMiddleware = applyMiddleware(
    ...middlewares
  )(create);

  const store = createStoreWithMiddleware(rootReducer, { parity: { path: parityPath } });

  if (module.hot) {
    module.hot.accept('../reducers', () => {
      const nextReducer = require('../reducers');
      store.replaceReducer(nextReducer);
    });
  }

  return store;
}
