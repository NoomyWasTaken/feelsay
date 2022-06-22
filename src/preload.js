// All of the Node.js APIs are available in the preload process.
// It has the same sandbox as a Chrome extension.
const { ipcRenderer, contextBridge, desktopCapturer } = require("electron");

// desktopCapturer.getSources({ types: ['window', 'screen'] }).then(async sources => {
//     for (const source of sources) {
//         if (source.name === 'Electron') {
//             mainWindow.webContents.send('SET_SOURCE', source.id)
//             return
//         }
//     }
// })

const API = {
    audioSources: (data) => ipcRenderer.invoke("sources/get", data).then((result) => {
        
    }),
    closeApp: () => ipcRenderer.send('closeApp'),
    minimizeApp: () => ipcRenderer.send('minimizeApp'),
    maximizeRestoreApp: () => ipcRenderer.send('maximizeRestoreApp'),
    start: () => ipcRenderer.send('start'),
}

// window.electronAPI
contextBridge.exposeInMainWorld("electronAPI", API);

// async function getAudioSources () {
//     desktopCapturer.getSources({ types: ['window', 'screen'] }).then(async sources => {
//         for (const source of sources) {
//             if (source.name === 'Electron') {
//                 mainWindow.webContents.send('SET_SOURCE', source.id)
//                 return
//             }
//         }
//     })
//     return 
// }

// ipcRenderer.on('SET_SOURCE', async (event, sourceId) => {
//     try {
//         const audioStream = await navigator.mediaDevices.getUserMedia({
//             audio: {
//                 mandatory: {
//                     chromeMediaSource: 'desktop'
//                 }
//             },
//             video: false
//         })
//         handleAudioStream(audioStream)
//     } catch (e) {
//         handleError(e)
//     }
// })

// function handleAudioStream (audioStream) {
//     //Handle the audio stream
//     console.log("Audio Handled")
// }

// function handleError (e) {
//     console.log(e)
// }