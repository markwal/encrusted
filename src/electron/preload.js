const { ipcRenderer, contextBridge } = require('electron')

contextBridge.exposeInMainWorld('appWindowManager', {
  close: () => { ipcRenderer.send("close") },
  minimize: () => { ipcRenderer.send("minimize") },
  maximize: () => { ipcRenderer.send("maximize") },
  unmaximize: () => { ipcRenderer.send("unmaximize") },
  onMaximize: (func) => { ipcRenderer.on("maximize", () => {func()}) },
  onUnmaximize: (func) => { ipcRenderer.on("unmaximize", () => {func()}) },
})

contextBridge.exposeInMainWorld('versions', process.versions)