document.onreadystatechange = (e) => {
    if (document.readyState == "complete") {
        setWindowControlHandlers();
    }
}

function setWindowControlHandlers() {
    // Make minimise/maximise/restore/close buttons work when they are clicked
    document.getElementById('min-button').addEventListener("click", event => {
        appWindowManager.minimize();
    });

    document.getElementById('max-button').addEventListener("click", event => {
        appWindowManager.maximize();
    });

    document.getElementById('restore-button').addEventListener("click", event => {
        appWindowManager.unmaximize();
    });

    document.getElementById('close-button').addEventListener("click", event => {
        appWindowManager.close()
    })

    appWindowManager.onMaximize(() => {console.log("did we add the style class?"); document.body.classList.add('maximized')})
    appWindowManager.onUnmaximize(() => {document.body.classList.remove('maximized')})
}