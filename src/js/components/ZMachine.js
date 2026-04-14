import React, { Component } from 'react';
import { connect } from 'react-redux';
import { useParams } from 'react-router-dom';
import { Group, Panel, Separator } from 'react-resizable-panels';

import ModalController from './ModalController';
import Settings from './Settings';
import Help from './Help';
import Transcript from './Transcript';
import DebugPanel from './DebugPanel';

function getDefaultLayout() {
  const raw = localStorage.getItem('setting:panel-layout');

  if (!raw) {
    return [65, 35];
  }

  try {
    const parsed = JSON.parse(raw);

    if (
      Array.isArray(parsed) &&
      parsed.length === 2 &&
      parsed.every(size => Number.isFinite(size))
    ) {
      return parsed;
    }
  } catch (err) {
    console.log('Error parsing panel layout:', err);
  }

  return [65, 35];
}

class ZMachine extends Component {
  constructor(props) {
    super(props);
    console.log("ZMachine constructor");
    this.showSettings = this.props.openModal.bind(this, <Settings />);
    this.showHelp = this.props.openModal.bind(this, <Help />);
    this.saveScreenDimensions = this.props.saveScreenDimensions.bind(this);
  }

  componentDidMount() {
    const vmScreenDimensions = {
      height: this.divElement.clientHeight,
      width: this.divElement.clientWidth,
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

    const defaultLayout = getDefaultLayout();

    return (
      <div className={containerName} ref={ (divElement) => { this.divElement = divElement } }>
        <ModalController />

        {!showPanel &&
          <Transcript filename={this.props.filename} />
        }

        {showPanel &&
          <Group
            orientation="horizontal"
            className="split-layout"
            defaultLayout={defaultLayout}
            onLayoutChanged={layout => localStorage.setItem('setting:panel-layout', JSON.stringify(layout))}
          >
            <Panel className="transcript-panel" minSize={40}>
              <Transcript filename={this.props.filename} />
            </Panel>

            <Separator className="split-resize-handle" />

            <Panel className="debug-panel-shell" minSize={20}>
              <DebugPanel />
            </Panel>
          </Group>
        }
      </div>
    );
  }
}


const ConnectedZMachine = connect(
  state => ({
    settings: state.settings,
  }),
  dispatch => ({
    openModal: child => dispatch({ type: 'MODAL::SHOW', child }),
    saveScreenDimensions: data  => dispatch({ type: 'INTERPRETER', data }),
  }),
)(ZMachine);

export default function RoutedZMachine() {
  const { filename } = useParams();

  return <ConnectedZMachine filename={filename} />;
}
