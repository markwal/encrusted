import React, { Component } from 'react';
import { connect } from 'react-redux';
import SplitPane from 'react-split-pane';

import ModalController from './ModalController';
import Settings from './Settings';
import Help from './Help';
import Transcript from './Transcript';
import DebugPanel from './DebugPanel';

function debounce(fn, delay) {
  let timeout;

  return function(...args) {
    clearTimeout(timeout);
    timeout = setTimeout(() => fn(...args), delay);
  };
}

class ZMachine extends Component {
  constructor(props) {
    super(props);
    this.showSettings = this.props.openModal.bind(this, <Settings />);
    this.showHelp = this.props.openModal.bind(this, <Help />);
    this.saveScreenDimensions = this.props.saveScreenDimensions.bind(this);
  }

  componentDidMount() {
    // FUTURE compute the fontSize from the monospace font
    const fontSize = parseInt(window.getComputedStyle(this.divElement).fontSize, 10);
    const vmScreenDimensions = {
      height: this.divElement.clientHeight,
      width: this.divElement.clientWidth,
      fontHeight: fontSize,
      fontWidth: Math.ceil(fontSize * 0.7),
    };
    this.saveScreenDimensions(vmScreenDimensions);
  }

  render() {
    const enabled = [
      this.props.settings.map,
      this.props.settings.tree,
      this.props.settings.instructions,
    ].filter(x => !!x);

    const showPanel = enabled.length > 0;
    const showTabs = enabled.length > 1;

    let containerName = (!showPanel)
      ? 'panel-hidden container'
      : 'container';

    if (showTabs) containerName += ' show-tabs';

    const str = localStorage.getItem('setting:size');
    const size = (str) ? parseInt(str, 10) : 700;
    const save = debounce(s => localStorage.setItem('setting:size', s), 500);

    return (
      <div className={containerName} ref={ (divElement) => { this.divElement = divElement } }>
        <ModalController />

        <SplitPane split="vertical" defaultSize={size} onChange={save}>
          <Transcript filename={this.props.match.params.filename} />
          <DebugPanel />
        </SplitPane>
      </div>
    );
  }
}


export default connect(
  state => ({
    settings: state.settings,
  }),
  dispatch => ({
    openModal: child => dispatch({ type: 'MODAL::SHOW', child }),
    saveScreenDimensions: data  => dispatch({ type: 'INTERPRETER', data }),
  }),
)(ZMachine);
