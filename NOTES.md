# Notes

## Web App Session Persistence

The web app persists state mostly through `localStorage`, scoped per story file,
plus IndexedDB for cached story files.

The main flow is:

1. `Transcript` loads the story file with `fileDB.load(filename)` and dispatches
   `TS::START` from `src/js/components/Transcript.js`.
2. The middleware creates `new LocalStore(filename)`, so persisted keys are
   namespaced by story file:
   - `<filename>::savestate`
   - `<filename>::script::0`
   - `<filename>::countScript`
   - `<filename>::map`
   - `<filename>::tree`
   - `<filename>::saves`
3. Rust emits a `savestate` before each read/input prompt in
   `src/rust/zmachine.rs` by calling `send_save_message("savestate", ...)`.
4. JS receives that in `src/js/middleware.js`, updates Redux, and writes it to
   localStorage as `<filename>::savestate`, but only after there has been real
   user input.
5. Every time the worker prints transcript HTML, middleware stores that move's
   rendered HTML separately as `<filename>::script::<moveNumber>` and updates
   `<filename>::countScript`.

On restore/page reload, `src/js/middleware.js` waits for the worker to load,
then checks either the `?save=` URL param or `<filename>::savestate`. If found,
it sends `load_savestate` to the wasm worker. The worker calls Rust
`load_savestate`, which restores VM memory, stack, and program counter without
treating it like an in-game `restore` command.

For the Transcript view specifically, the app does not restore a rich React view
state object. It reconstructs the visible transcript by concatenating the stored
`script::<n>` HTML fragments into one large transcript blob, stores that back as
`script::0`, then dispatches `TS::TEXT` with that HTML. `Move` renders it via
`dangerouslySetInnerHTML`.

Nuances:

- Scroll position is not restored. `Transcript.componentDidUpdate()` always
  scrolls to the bottom.
- Command history is not fully restored as Redux `history`; old commands are
  visible in the transcript HTML, but the up-arrow command history array starts
  fresh.
- Split panel layout is a separate global UI setting stored as
  `setting:panel-layout` in `src/js/components/ZMachine.js`.
- Story files themselves are cached in IndexedDB by `src/js/fileDB.js`, not
  localStorage.

## Undo, Redo, And Transcript Chunks

The browser transcript is stored in Redux as `state.transcript`, with `moves`,
`undos`, `history`, `header`, and `quit` fields. Each visible transcript chunk
is a `move` object shaped like `{ text, input, location }`. A worker `print`
event dispatches `TS::TEXT`, which appends a new move with the printed HTML in
`text` and empty `input` and `location` fields.

When the player submits a command, `TS::SUBMIT` trims the input and attaches it
to the most recent move. The command is not stored as its own transcript entry;
it is grafted onto the current output chunk, then rendered by `Move` either as
the active input field or as prior user input text.

UI undo and redo manipulate the transcript stacks only:

- `TS::UNDO` moves the last item from `moves` to `undos`.
- `TS::REDO` moves the last item from `undos` back to `moves`.

When a move has a stored location, the transcript reducer also writes that
location into `header.left`. That keeps the visible header synchronized with
the active move when undoing, redoing, and restoring persisted transcript text.

The middleware also sends `undo` and `redo` messages to the wasm worker. The
worker calls `Engine.undo()` or `Engine.redo()`, which delegate to the Rust
`Zmachine`. Rust keeps the real VM history as serialized Quetzal states:
`current_state: Option<(String, Vec<u8>)>`, `undos: Vec<(String, Vec<u8>)>`,
and `redos: Vec<(String, Vec<u8>)>`. Undo pops a prior state from `undos`,
pushes the current state into `redos`, restores the popped bytes, and makes that
state current. Redo does the reverse.

During a runtime browser session, transcript chunks live in memory as Redux
`moves`. They are also persisted per story file in `localStorage` as
`<filename>::script::<n>` keys, with `<filename>::countScript` tracking the
latest chunk number. Each stored script chunk contains the last input label plus
the rendered transcript HTML from that print event. The move location is stored
next to it as `<filename>::script-location::<n>` once the worker's map update
reports the current room.

On reload from a saved runtime state, middleware reassembles the stored
`script::<n>` entries into one large transcript HTML string, stores it back as
`script::0`, carries forward the latest `script-location::<n>` as
`script-location::0`, removes the old numbered chunks, starts the VM, then
dispatches that accumulated HTML and restored location as one `TS::TEXT`.
