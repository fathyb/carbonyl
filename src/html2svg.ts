import { app, BrowserWindow } from 'electron'

app.dock.hide()
// app.commandLine.appendSwitch('enable-logging')
app.commandLine.appendSwitch('headless')
app.whenReady()
    .then(async () => {
        const page = new BrowserWindow({
            show: false,
            width: 1920,
            height: 1080,
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

                        await page.loadURL('https://yari-demos.prod.mdn.mozit.cloud/en-US/docs/Web/API/Canvas_API/Tutorial/Drawing_shapes/_sample_.making_combinations.html')
                    })
                    .catch(reject),
            )

            return await page.webContents.executeJavaScript(
                `
                    new Promise(resolve => {
                        const style = document.createElement('style')

                        style.innerHTML = \`
                            body::-webkit-scrollbar, body::-webkit-scrollbar-track, body::-webkit-scrollbar-thumb {
                                display: none;
                            }
                        \`

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
    .then((result) => {
        console.log(result)

        process.exit(0)
    })
    .catch((error) => {
        console.error(error)

        process.exit(1)
    })
