// All of the Node.js APIs are available in the preload process.
// It has the same sandbox as a Chrome extension.
const { ipcRenderer, contextBridge, desktopCapturer } = require("electron");

// API - communicate with render process
const API = {
    audioSources: (data) => ipcRenderer.invoke("sources/get", data).then((result) => {

    }),
    closeApp: () => ipcRenderer.send('closeApp'),
    minimizeApp: () => ipcRenderer.send('minimizeApp'),
    maximizeRestoreApp: () => ipcRenderer.send('maximizeRestoreApp'),
    start: () => ipcRenderer.send('start'),
}

// window.electronAPI - communicate with main process
contextBridge.exposeInMainWorld("electronAPI", API);

// catch from index.js
ipcRenderer.on('SET_SOURCE', async (event, source) => {
    try {
        const stream = await navigator.mediaDevices.getUserMedia({
            audio: true,
            video: false
        })
        handleStream(stream)
    } catch (e) {
        handleError(e)
    }
})

// take the audio stream from the app and use the SpeechRecognition web API
function handleStream (stream) {
    const texts = document.querySelector('.texts');
  
    // SpeechRecognition web API
    window.SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
  
    const recognition = new window.SpeechRecognition();
    
    // interimResults returns results in real time, set to false to wait for speech to end first (isFinal == true)
    recognition.interimResults = true;
    recognition.continuous = true;
    recognition.lang = 'en-US';
  
    // create an h1 element to display the text response
    let h1 = document.createElement('h1');
  
    // listener for the result, adds result to paragraph for displaying
    recognition.addEventListener('result', (e) => {
        const text = Array.from(e.results)
            .map(result => result[0])
            .map(result => result.transcript)
            .join('');

        h1.innerText = text;
        texts.appendChild(h1);

        // checks if there is a pause in speech, if so create new paragraph
        if (e.results[0].isFinal) {
            h1 = document.createElement('h1');
        }
    });
  
    recognition.start();
  }