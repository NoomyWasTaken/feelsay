# Read Me

This app is part of the undergraduate BSc Thesis in Computer Science by Neuman Alkhalil.

Before running install all dependencies in the **package.json** file.

The app currently does not function due to limitations in obtaining audio data via the navigator.mediaDevices.getUserMedia API:
> https://github.com/w3c/mediacapture-main/issues/650

The speech recognition API's used in this app are the default browser Web Speech API:
>https://developer.mozilla.org/en-US/docs/Web/API/Web_Speech_API/Using_the_Web_Speech_API

Limitations of this API are that it only takes microphone as input

The other API used (main speech-to-text API) is found within python file **real-time-sr.py** which utilizes the AssemblyAI speech-to-text API:
>https://app.assemblyai.com/

The file contains a private API key used to access resources and I will therefore ask any holders of the code not to share it to other unauthorized persons.