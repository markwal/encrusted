import React, { Component } from 'react';
import { connect } from 'react-redux';


class Header extends Component {
  constructor(props) {
    super(props);

    this.title = window.location.pathname.split('/').pop();
    this.title = this.title.charAt(0).toUpperCase() + this.title.slice(1);
    this.saveScreenDimensions = this.props.saveScreenDimensions.bind(this);
  }

  componentDidMount() {
    // compute the size of the monospace emspace
    const vmScreenDimensions = {
      font_height: this.divElement.clientHeight,
      font_width: Math.ceil(this.divElement.clientWidth / 10),
    };
    console.log('the emspace width ', vmScreenDimensions.font_width, ' and height ', vmScreenDimensions.font_height);
    this.saveScreenDimensions(vmScreenDimensions);
  }

  render() {
    document.title = (this.props.left) ? `${this.title} - ${this.props.left}` : this.title;

    return (
      <div className="header">
        <div>
          <div className="left">
            {this.props.canUndo &&
              <i className="undo icon ion-chevron-left" onClick={this.props.undo}></i>
            }

            {this.props.left || '\u00A0'}
            <div className="test-font" ref={ divElement => { this.divElement = divElement } }>
              {'I'.repeat(10)}
            </div>
          </div>

          <div className="right">
            {this.props.canRedo &&
              <i className="redo icon ion-chevron-right" onClick={this.props.redo}></i>
            }
            {this.props.right || '\u00A0'}
          </div>
        </div>
      </div>
    );
  }
}


export default connect(
  state => ({
    left: state.transcript.header.left,
    right: state.transcript.header.right,
    canUndo: !state.transcript.quit && state.transcript.moves.length > 1,
    canRedo: !state.transcript.quit && !!state.transcript.undos.length,
  }),
  dispatch => ({
    undo: () => dispatch({ type: 'TS::UNDO' }),
    redo: () => dispatch({ type: 'TS::REDO' }),
    saveScreenDimensions: data => dispatch({ type: 'INTERPRETER', data }),
  }),
)(Header);
