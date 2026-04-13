const {
  app,
  BrowserWindow,
  ipcMain,
  nativeTheme,
  shell
} = require('electron')
const path = require('path')

let electronWindow;

function isExternalHttpUrl(url) {
  const parsed = new URL(url);
  return parsed.protocol === 'http:' || parsed.protocol === 'https:';
}

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

  electronWindow.webContents.setWindowOpenHandler(({ url }) => {
    if (isExternalHttpUrl(url)) {
      shell.openExternal(url)
      return { action: 'deny' }
    }

    return { action: 'allow' }
  });

  electronWindow.webContents.on('will-navigate', function(e, url) {
    if (isExternalHttpUrl(url)) {
      e.preventDefault()
      shell.openExternal(url)
    }
  })

  // watch for maximize state change
  electronWindow.on('maximize', () => { electronWindow.webContents.send('maximize') });
  electronWindow.on('unmaximize', () => { electronWindow.webContents.send('unmaximize') });
  // trigger one now for initial state
  electronWindow.webContents.once('did-finish-load', () => {
    electronWindow.webContents.send(
      electronWindow.isMaximized() ? 'maximize' : 'unmaximize'
    );
  });

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
