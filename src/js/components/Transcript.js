import React, { Component } from 'react';
import { connect } from 'react-redux';

import fileDB from '../fileDB';
import Move from './Move';
import Header from './Header';
import ErrorModal from './ErrorModal';


class Transcript extends Component {
  constructor(props) {
    super(props);

    this.submit = this.submit.bind(this);
    this.undo = this.undo.bind(this);
    this.redo = this.redo.bind(this);

    const filename = props.filename;

    const onErr = (err, msg) => {
      this.props.openModal(<ErrorModal err={err} msg={msg} />);
    };

    fileDB.load(filename)
      .then(
        file => this.props.start(filename, file),
        err => onErr(err, `Error getting story file: ${err}`),
      )
      .catch(err => onErr(err, `Unknown start up error: ${err}`));
  }

  componentWillUnmount() {
    this.props.stop();
  }

  static getDerivedStateFromProps() {
    // child `Move` elements might talk, here we cancel when there is a change
    window.speechSynthesis && window.speechSynthesis.cancel();
    return null;
  }

  componentDidUpdate() {
    requestAnimationFrame(() => {
      this.el.scrollTop = this.el.scrollHeight;
    });
  }

  submit(input) {
    this.props.submit(input.trim());
  }

  undo() {
    if (this.props.moves.length > 1) this.props.undo();
  }

  redo() {
    if (this.props.canRedo) this.props.redo();
  }

  render() {
    // only the last move is "active" and will have a text input field
    const moves = this.props.moves.map((move, index) => (
      <Move
        isActive={(index === this.props.moves.length - 1)}
        undo={this.undo}
        redo={this.redo}
        submit={this.submit}
        history={this.props.history}
        isRead={this.props.isRead}
        key={index}
        {...move}
      />
    ));

    // fade in transcript at start
    let className = (moves.length) ? 'show transcript' : 'hide transcript';
    // opt remove styling on objects/rooms
    if (this.props.isPlain) className += ' plain';
    if (this.props.isMonospace) className += ' monospace';

    return (
      <div className={className} ref={el => this.el = el}>
        <Header/>

        <div className="moves">
          {moves}
        </div>

        <div className="bottom-dummy"></div>
      </div>
    );
  }
}


export default connect(
  state => ({
    moves: state.transcript.moves,
    history: state.transcript.history,
    canRedo: !!state.transcript.undos.length,
    saves: state.saves.saves,
    isRead: state.settings.read,
    isPlain: state.settings.plain,
    isMonospace: state.settings.mono,
  }),
  dispatch => ({
    start: (filename, file) => dispatch({ type: 'TS::START', filename, file }),
    submit: input => dispatch({ type: 'TS::SUBMIT', input }),
    undo: () => dispatch({ type: 'TS::UNDO' }),
    redo: () => dispatch({ type: 'TS::REDO' }),
    stop: () => dispatch({ type: 'TS::STOP' }),
    openModal: child => dispatch({ type: 'MODAL::SHOW', child }),
  }),
)(Transcript);
