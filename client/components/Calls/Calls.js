import React, { Component, PropTypes } from 'react';
import AnimateChildren from '../../components-compositors/Animated/children';
import Call from '../Call';
import CallsToolbar from '../CallsToolbar';
import styles from './style.css';

export default class Calls extends Component {

  state = {
    activeCall: null,
    activeChild: null
  }

  render () {
    return (
      <div
        className='calls-container'
        onMouseLeave={this.clearActiveCall}
        {...this._test('container')}
      >
        {this.renderClear()}
        <h2 className={styles.header}>History</h2>
        <div className={`${styles.history} row`} ref={this.setCallsHistory}>
          {this.renderNoCallsMsg()}
          {this.renderCalls()}
        </div>
        <CallsToolbar
          call={this.state.activeCall}
          callEl={this.state.activeChild}
          containerEl={this._callsHistory}
          actions={this.props.actions}
        />
      </div>
    );
  }

  renderClear () {
    if (!this.props.calls.length) {
      return;
    }

    return (
      <a
        {...this._test('remove')}
        title='Clear RPC calls history'
        onClick={this.clearHistory}
        className={styles.removeIcon}
        >
        <i className='icon-trash'></i>
      </a>
    );
  }

  renderNoCallsMsg () {
    if (this.props.calls.length) {
      return;
    }

    return (
      <AnimateChildren>
        <div {...this._test('empty-wrapper')}>
          <h3 className={styles.historyInfo} {...this._test('empty')}>
            Fire up some calls and the results will be here.
          </h3>
        </div>
      </AnimateChildren>
    );
  }

  renderCalls () {
    const { calls } = this.props;

    if (!calls.length) {
      return;
    }

    return (
      <AnimateChildren>
        {calls.map((call, idx) => (
          <Call
            key={calls.length - idx}
            call={call}
            setActiveCall={this.setActiveCall}
          />
        ))}
      </AnimateChildren>
    );
  }

  clearActiveCall = () => {
    this.setState({ activeCall: null, activeElement: null });
  }

  setActiveCall = (call, el) => {
    this.setState({ activeCall: call, activeElement: el });
  }

  setCallsHistory = (el) => {
    this._callsHistory = el;
  }

  clearHistory = () => {
    this.props.reset();
  }

  static propTypes = {
    calls: PropTypes.arrayOf(PropTypes.object).isRequired,
    actions: PropTypes.shape({
      fireRpc: PropTypes.func.isRequired,
      copyToClipboard: PropTypes.func.isRequired,
      selectRpcMethod: PropTypes.func.isRequired
    }).isRequired,
    reset: PropTypes.func
  }

}
