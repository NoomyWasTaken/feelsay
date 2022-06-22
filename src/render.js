closeBtn.addEventListener('click', () => {
    electronAPI.closeApp();
    console.log("Clicked close");
})

minimizeBtn.addEventListener('click', () => {
    electronAPI.minimizeApp();
    console.log("Clicked minimize");
})

maxResBtn.addEventListener('click', () => {
    electronAPI.maximizeRestoreApp();
    console.log("Clicked maximize restore");
})

startBtn.addEventListener('click', () => {
    electronAPI.start();
    console.log("Clicked on start");
})

targetBtn.addEventListener('click', () => {
    getAudioSources();
})

// // Buttons
// const targetBtn = document.getElementById('targetBtn');
// const startBtn = document.getElementById('startBtn');
// targetBtn.onclick = getAudioSources;

// //const { desktopCapturer } = require('electron');
// //const { Menu } = require('@electron/remote');

// //Get the available audio sources
async function getAudioSources() {
    const sources = await electronAPI.audioSources();
    setTimeout(console.log(sources), 5000);
    
}