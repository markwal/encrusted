const {
  app,
  BrowserWindow,
  ipcMain,
  nativeTheme,
  shell
} = require('electron')
const path = require('path')

let electronWindow;

function createWindow() {
  electronWindow = new BrowserWindow({
    width: app.isPackaged ? 800 : 1200,
    height: 800,
    frame: false,
    titleBarStyle: 'hidden',
    backgroundColor: (nativeTheme.shouldUseDarkColors ? '#000' : '#FFF'),
    webPreferences: {
      preload: path.join(__dirname, './preload.js'),
      contextIsolation: true
    }
  })

  electronWindow.loadURL(`file://${path.join(__dirname, 'index.html')}`)

  electronWindow.webContents.on('new-window', function(e, url) {
    u = new URL(url)
    console.log("new-window u.hostname ", u.hostname)
    console.log("new-window u.pathname ", u.pathname)
    console.log("new-window u.protocol ", u.protocol)
    e.preventDefault()
    shell.openExternal(url)
  })

  electronWindow.webContents.on('will-navigate', function(e, url) {
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
  electronWindow.webContents.setWindowOpenHandler(function({ url }) {
    u = new URL(url)
    if (u.protocol === 'http:' || u.protocol === 'https:') {
      shell.openExternal(url)
      return { action: 'deny' }
    }
    return { action: 'allow', overrideBrowserWindowOptions: { modal: true } }
  });
  */

  // watch for maximize state change
  electronWindow.on('maximize', () => { electronWindow.webContents.send('maximize') });
  electronWindow.on('unmaximize', () => { electronWindow.webContents.send('unmaximize') });
  // trigger one now for initial state
  electronWindow.webContents.send(electronWindow.isMaximized ? "maximize" : "unmaximize")

  if (!app.isPackaged) {
    electronWindow.webContents.openDevTools()
  }
 
  ipcMain.on('close', () => { electronWindow.close(); })
  ipcMain.on('minimize', () => { electronWindow.minimize(); })
  ipcMain.on('maximize', () => { electronWindow.maximize(); })
  ipcMain.on('unmaximize', () => { electronWindow.unmaximize(); })
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
