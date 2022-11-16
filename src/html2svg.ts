import { program } from 'commander'
import { app, BrowserWindow } from 'electron'

const entry = process.argv.find((a) => a.endsWith('html2svg.js'))
const index = entry ? process.argv.indexOf(entry) : -1
const args = process.argv.slice(Math.max(2, index + 1))

program
    .name('html2svg')
    .showHelpAfterError()
    .showSuggestionAfterError()
    .argument('<url>', 'URL to the web page to render')
    .option('-f, --full', 'capture the entire page')
    .option(
        '-w, --wait <seconds>',
        'amount of time to wait between the page loaded event and taking the screenshot',
        validateInt,
        1,
    )
    .option(
        '-w, --width <width>',
        'set the viewport width in pixels',
        validateInt,
        1920,
    )
    .option(
        '-h, --height <height>',
        'set the viewport height in pixels',
        validateInt,
        1080,
    )
    .option(
        '-f, --format <format>',
        'set the output format, should one of these values: svg, pdf',
        'svg',
    )
    .action(async (url, { full, wait, width, height, format }) => {
        const mode = getMode(format)

        app.dock?.hide()
        app.commandLine.appendSwitch('headless')
        app.commandLine.appendSwitch('no-sandbox')
        app.commandLine.appendSwitch('disable-gpu')

        await app.whenReady()

        const page = new BrowserWindow({
            width,
            height,
            show: false,
            webPreferences: { sandbox: false },
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

            await page.webContents.executeJavaScript(
                `
                    new Promise(resolve => {
                        const style = document.createElement('style')

                        style.innerHTML = trustedTypes
                            .createPolicy('html2svg/scrollbar-css', { createHTML: x => x })
                            .createHTML(\`
                                *::-webkit-scrollbar,
                                *::-webkit-scrollbar-track,
                                *::-webkit-scrollbar-thumb {
                                    display: none;
                                }
                            \`)

                        document.head.appendChild(style)
                        scrollTo({ top: document.body.scrollHeight })

                        requestAnimationFrame(() => {
                            scrollTo({ top: 0 })

                            requestAnimationFrame(() =>
                                setTimeout(resolve, ${wait})
                            )
                        })
                    }).then(() =>
                        getPageContentsAsSVG(
                            ${full ? 0 : height} * devicePixelRatio,
                            ${mode},
                            document.title,
                        )
                    )
                `,
            )
        } finally {
            page.destroy()
        }

        process.exit(0)
    })
    .parseAsync(args, { from: 'user' })
    .catch((error) => {
        console.error(error)

        process.exit(1)
    })

function getMode(format: string) {
    switch (format) {
        case 'svg':
            return 0
        case 'pdf':
            return 1
        default:
            throw new Error(`Unsupported output format: ${format}`)
    }
}

function validateInt(string: string) {
    const number = parseInt(string, 10)

    if (Number.isNaN(number)) {
        throw new Error(`Invalid number value: ${string}`)
    }

    return number
}
