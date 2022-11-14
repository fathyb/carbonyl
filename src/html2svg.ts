import { app, BrowserWindow } from 'electron'


Promise.resolve().then(async () => {
    const entry = process.argv.find(a => a.endsWith('html2svg.js'))
    const index = entry ? process.argv.indexOf(entry) : -1
    const args = process.argv.slice(Math.max(2, index + 1))
    const [url] = args
    
    if (!url) {
        throw new Error('Usage: html2svg [url]')
    }
    
    app.dock?.hide()
    app.commandLine.appendSwitch('headless')
    app.commandLine.appendSwitch('no-sandbox')
    app.commandLine.appendSwitch('disable-gpu')

    await app.whenReady()

    return url
})
    .then(async (url) => {
        const page = new BrowserWindow({
            show: false,
            width: 1920,
            height: 1080,

            webPreferences: {
                sandbox: false,
                webSecurity: false,
            }
        })

        try {
            await new Promise<void>((resolve, reject) =>
                Promise.resolve()
                    .then(async () => {
                        const timeout = setTimeout(() => {
                            page.webContents.off('did-finish-load', listener)

                            reject(new Error('timeout'))
                        }, 10_000)
                        const listener = () => {
                            clearTimeout(timeout)

                            resolve()
                        }

                        page.webContents.once('did-finish-load', listener)

                        await page.loadURL(url)
                    })
                    .catch(reject),
            )

            return await page.webContents.executeJavaScript(
                `
                    new Promise(resolve => {
                        const style = document.createElement('style')
                        const policy = trustedTypes.createPolicy('html2svg/scrollbar-css', { createHTML: x => x })

                        style.innerHTML = policy.createHTML(\`
                            body::-webkit-scrollbar, body::-webkit-scrollbar-track, body::-webkit-scrollbar-thumb {
                                display: none;
                            }
                        \`)

                        document.head.appendChild(style)
                        scrollTo({ top: document.body.scrollHeight })

                        requestAnimationFrame(() => {
                            scrollTo({ top: 0 })

                            setTimeout(() => {
                                requestAnimationFrame(resolve)
                            }, 1000)
                        })
                    }).then(getPageContentsAsSVG)
                `,
            )
        } finally {
            page.destroy()
        }
    })
    .then(async (result) => {
        await print(result)

        process.exit(0)
    })
    .catch((error) => {
        console.error(error)

        process.exit(1)
    })

// Electron seems to drop lines if we send them too fast on slow streams like Docker..
async function print(output: string) {
    const awfulBugSizeHeuristic = 1024

    for(let i = 0; i < output.length; i += awfulBugSizeHeuristic) {
        await new Promise<void>((resolve, reject) =>
            process.stdout.write(
                output.slice(i, i + awfulBugSizeHeuristic),
                error => error ? reject(error) : resolve(),
            )
        )
    }
}
