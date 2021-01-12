const { contextBridge } = require('electron')

contextBridge.exposeInMainWorld('myAPI', {
  something: () => { return "hey there" }
})

contextBridge.exposeInMainWorld('versions', process.versions)
