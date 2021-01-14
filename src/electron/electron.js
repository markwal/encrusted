const { app, BrowserWindow, nativeTheme, shell } = require('electron')
const path = require('path')

function createWindow() {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    backgroundColor: (nativeTheme.shouldUseDarkColors ? '#000' : '#FFF'),
    webPreferences: {
      contextIsolation: true
    }
  })

  win.loadURL(`file://${path.join(__dirname, 'index.html')}`)

  win.webContents.on('new-window', function(e, url) {
    u = new URL(url)
    console.log("new-window u.hostname ", u.hostname)
    console.log("new-window u.pathname ", u.pathname)
    console.log("new-window u.protocol ", u.protocol)
    e.preventDefault()
    shell.openExternal(url)
  })

  win.webContents.on('will-navigate', function(e, url) {
    console.log("on will-navigate")

    u = new URL(url)
    console.log("u.hostname ", u.hostname)
    console.log("u.pathname ", u.pathname)
    console.log("u.protocol ", u.protocol)
    if (u.protocol === 'http:' || u.protocol === 'https:') {
      e.preventDefault()
      shell.openExternal(url)
    }
  })

  /* FUTURE when on 'new-window' is deprecated
  win.webContents.setWindowOpenHandler(function({ url }) {
    u = new URL(url)
    if (u.protocol === 'http:' || u.protocol === 'https:') {
      shell.openExternal(url)
      return { action: 'deny' }
    }
    return { action: 'allow', overrideBrowserWindowOptions: { modal: true } }
  });
  */

  if (!app.isPackaged) {
    win.webContents.openDevTools()
  }
}

app.whenReady().then(createWindow)

app.on('window-all-closed', () => {
  if (process.platform != 'darwin') {
    app.quit()
  }
})

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow()
  }
})
