// buttons
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

// open new window and start speech recognition
startBtn.addEventListener('click', () => {
    electronAPI.start();
    console.log("Clicked start");
})

// get the available audio sources
targetBtn.addEventListener('click', () => {
    getAudioSources();
})

async function getAudioSources() {
    const sources = await electronAPI.audioSources();
}