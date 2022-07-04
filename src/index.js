const { app, BrowserWindow, ipcMain, desktopCapturer, Menu } = require('electron');
const path = require('path');

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
  // eslint-disable-line global-require
  app.quit();
}

const createWindow = () => {
  // Create the browser window.
  const mainWindow = new BrowserWindow({
    icon: path.join(__dirname, "FeelSay Icon.ico"),
    width: 900,
    height: 600,
    minWidth: 800,
    minHeight: 500,
    frame: false,
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, './preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      // devTools: false,
    }
  });

  // and load the index.html of the app.
  mainWindow.loadFile(path.join(__dirname, 'index.html'));

  // Open the DevTools. Uncomment to use
  // mainWindow.webContents.openDevTools();

  ipcMain.on('closeApp', () => {
    console.log('clicked on close btn');
    mainWindow.close();
  });

  ipcMain.on('minimizeApp', () => {
    console.log('clicked on minimize btn');
    mainWindow.minimize();
  });

  ipcMain.on('maximizeRestoreApp', () => {
    console.log('clicked on maximize restore btn');
    if(mainWindow.isMaximized()) {
      mainWindow.restore();
    } else {
      mainWindow.maximize();
    }
  });

  ipcMain.handle("sources/get", async (event, data) => {
    const inputSources = await desktopCapturer.getSources({
      types: ['window', 'desktop']
    });

    const audioOptionsMenu = Menu.buildFromTemplate(
      inputSources.map(source => {
        return {
          label: source.name,
          click: () => {
            console.log(source);
            mainWindow.webContents.send('SET_SOURCE', source)
            return source;
          }
        };
      })
    );

    audioOptionsMenu.popup();
  });

};

const createSubtitleWindow = () => {
  const subtitleWindow = new BrowserWindow({
    icon: path.join(__dirname, "FeelSay Icon.ico"),
    width: 600,
    height: 75,
    minWidth: 300,
    minHeight: 50,
    frame: false,
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, './preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      // devTools: false,
    }
  });

  subtitleWindow.setAlwaysOnTop(true, 'screen-saver')

  ipcMain.on('closeApp', () => {
    console.log('clicked on close btn');
    subtitleWindow.close();
  });

  subtitleWindow.loadFile(path.join(__dirname, 'subtitle.html'));
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow);

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  // On OS X it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and import them here.

ipcMain.on('start', createSubtitleWindow);